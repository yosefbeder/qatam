use super::ast::{Expr, Literal, Stml};
use super::operators::{Associativity, OPERATORS};
use super::reporter::{Phase, Report, Reporter};
use super::token::{Token, TokenType, INVALID_TYPES};
use super::tokenizer::Tokenizer;
use std::rc::Rc;

pub struct Parser<'a, 'b, 'c> {
    tokenizer: &'a mut Tokenizer<'b>,
    current: Option<Token<'b>>,
    previous: Option<Token<'b>>,
    reporter: &'c mut dyn Reporter<'b>,
}

impl<'a, 'b, 'c> Parser<'a, 'b, 'c> {
    pub fn new(tokenizer: &'a mut Tokenizer<'b>, reporter: &'c mut dyn Reporter<'b>) -> Self {
        Self {
            tokenizer,
            current: None,
            previous: None,
            reporter,
        }
    }

    fn check_current(&self) -> Result<(), ()> {
        match &self.current {
            Some(token) => {
                if INVALID_TYPES.contains(&token.typ) {
                    return Err(());
                }
                return Ok(());
            }
            None => unreachable!(),
        }
    }

    fn advance(&mut self) -> Result<(), ()> {
        match &self.current {
            Some(token) => {
                if token.typ == TokenType::EOF {
                    return Ok(());
                }
            }
            None => {}
        };

        self.previous = self.current.clone();
        let mut next_token = self.tokenizer.next_token(self.reporter);

        loop {
            if next_token.typ == TokenType::Comment {
                next_token = self.tokenizer.next_token(self.reporter);
                continue;
            }
            break;
        }
        self.current = Some(next_token);

        self.check_current()?;

        Ok(())
    }

    fn next(&mut self) -> Result<Token<'b>, ()> {
        self.advance()?;
        match &self.previous {
            Some(token) => Ok(token.clone()),
            None => unreachable!(),
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &'static str) -> Result<(), ()> {
        if self.check(typ) {
            self.advance()?;
            Ok(())
        } else {
            let report = Report::new(
                Phase::Parsing,
                msg.to_string(),
                match &self.current {
                    Some(token) => token.clone(),
                    None => unreachable!(),
                },
            );
            self.reporter.error(report);
            Err(())
        }
    }

    fn peek(&self) -> Token<'b> {
        match &self.current {
            Some(token) => token.clone(),
            None => unreachable!(),
        }
    }

    fn check(&self, typ: TokenType) -> bool {
        if self.peek().typ == typ {
            return true;
        }

        false
    }

    fn at_end(&self) -> bool {
        self.check(TokenType::EOF)
    }

    fn exprs(&mut self) -> Result<Vec<Expr<'b>>, ()> {
        let mut items = vec![self.expr(9, true)?];
        while self.check(TokenType::Comma) {
            self.advance()?;
            if self.check(TokenType::CBracket) || self.check(TokenType::CParen) {
                break;
            }
            items.push(self.expr(9, true)?);
        }
        Ok(items)
    }

    fn list(&mut self) -> Result<Expr<'b>, ()> {
        let items = if self.check(TokenType::CBracket) {
            vec![]
        } else {
            self.exprs()?
        };

        self.consume(TokenType::CBracket, "توقعت ']' بعد القائمة")?;

        Ok(Expr::Literal(Literal::List(items)))
    }
    fn property(&mut self) -> Result<(Rc<Token<'b>>, Expr<'b>), ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الخاصية")?;
        let key = match &self.previous {
            Some(token) => token.clone(),
            None => unreachable!(),
        };
        self.consume(TokenType::Colon, "توقعت ':' بعد الاسم")?;
        Ok((Rc::new(key), self.expr(9, true)?))
    }

    fn object(&mut self) -> Result<Expr<'b>, ()> {
        let mut items;
        if self.check(TokenType::CBrace) {
            items = vec![]
        } else {
            items = vec![self.property()?];
            while self.check(TokenType::Comma) {
                self.advance()?;
                if self.check(TokenType::CBrace) {
                    break;
                }
                items.push(self.property()?);
            }
        };

        self.consume(TokenType::CBrace, "توقعت '}' بعد القائمة")?;

        Ok(Expr::Literal(Literal::Object(items)))
    }

    fn literal(&mut self) -> Result<Expr<'b>, ()> {
        let token = match &self.previous {
            Some(token) => token.clone(),
            None => unreachable!(),
        };

        match token.typ {
            TokenType::Identifier => Ok(Expr::Variable(Rc::new(token.clone()))),
            TokenType::Number => Ok(Expr::Literal(Literal::Number(Rc::new(token.clone())))),
            TokenType::String => Ok(Expr::Literal(Literal::String(Rc::new(token.clone())))),
            TokenType::True | TokenType::False => {
                Ok(Expr::Literal(Literal::Bool(Rc::new(token.clone()))))
            }
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil(Rc::new(token.clone())))),
            TokenType::OBracket => self.list(),
            TokenType::OBrace => self.object(),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self) -> Result<Expr<'b>, ()> {
        let token = match &self.previous {
            Some(token) => token.clone(),
            None => unreachable!(),
        };

        let row: usize = token.typ.into();
        let prefix_precedence = match OPERATORS[row].0 {
            Some(precedence) => precedence,
            None => unreachable!(),
        };
        let right = self.expr(prefix_precedence, false)?;
        Ok(Expr::Unary(Rc::new(token), Box::new(right)))
    }

    fn group(&mut self) -> Result<Expr<'b>, ()> {
        let expr = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' لإغلاق المجموعة")?;

        return Ok(expr);
    }

    /// Parses any expression with a binding power more than or equal to `min_bp`.
    fn expr(&mut self, min_precedence: u8, mut can_assign: bool) -> Result<Expr<'b>, ()> {
        //                                 ^^^ I coulnd't find a better approach :)
        let mut token = self.next()?;
        let mut expr;

        expr = match token.typ {
            TokenType::Identifier
            | TokenType::Number
            | TokenType::String
            | TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::OBracket
            | TokenType::OBrace => self.literal()?,
            TokenType::Minus | TokenType::Bang => self.unary()?,
            TokenType::OParen => self.group()?,
            _ => {
                let report = Report::new(Phase::Parsing, "توقعت عبارة".to_string(), token.clone());
                self.reporter.error(report);
                return Err(());
            }
        };

        while !self.at_end() {
            token = self.peek();

            let row: usize = token.typ.into();

            if let Some(infix_precedence) = OPERATORS[row].1 {
                let associativity = OPERATORS[row].3.unwrap();

                if min_precedence < infix_precedence {
                    break;
                }

                if token.typ != TokenType::Equal {
                    can_assign = false;
                }

                self.advance()?;

                if token.typ == TokenType::Equal && !can_assign {
                    let report = Report::new(
                        Phase::Parsing,
                        "الجانب الأيمن غير صحيح".to_string(),
                        token.clone(),
                    );
                    self.reporter.error(report);
                    return Err(());
                }

                expr = Expr::Binary(
                    Rc::new(token),
                    Box::new(expr),
                    Box::new(self.expr(
                        match associativity {
                            Associativity::Right => infix_precedence,
                            Associativity::Left => infix_precedence - 1,
                        },
                        can_assign,
                    )?),
                );
            } else if let Some(postfix_precedence) = OPERATORS[row as usize].2 {
                if min_precedence < postfix_precedence {
                    break;
                }
                self.advance()?;

                match token.typ {
                    TokenType::OParen => {
                        let args = if self.check(TokenType::CParen) {
                            vec![]
                        } else {
                            self.exprs()?
                        };
                        self.consume(TokenType::CParen, "توقعت ')' بعد القائمة")?;

                        expr = Expr::Call(Rc::new(token), Box::new(expr), args);
                    }
                    //TODO>> abstract
                    TokenType::Period => {
                        self.consume(TokenType::Identifier, "توقعت اسم الخاصية")?;
                        let key = Expr::Variable(Rc::new(match &self.previous {
                            Some(token) => token.clone(),
                            None => unreachable!(),
                        }));

                        if self.check(TokenType::Equal) {
                            token = self.next()?;
                            if !can_assign {
                                let report = Report::new(
                                    Phase::Parsing,
                                    "الجانب الأيمن غير صحيح".to_string(),
                                    token.clone(),
                                );
                                self.reporter.error(report);
                                return Err(());
                            }

                            expr = Expr::Set(
                                Rc::new(token),
                                Box::new(expr),
                                Box::new(key),
                                Box::new(self.expr(postfix_precedence, true)?),
                            );
                        } else {
                            expr = Expr::Get(Rc::new(token), Box::new(expr), Box::new(key));
                        }
                    }
                    TokenType::OBracket => {
                        let key = self.expr(9, true)?;
                        self.consume(TokenType::CBracket, "توقعت ']' بعد العبارة")?;
                        if self.check(TokenType::Equal) {
                            token = self.next()?;
                            if !can_assign {
                                let report = Report::new(
                                    Phase::Parsing,
                                    "الجانب الأيمن غير صحيح".to_string(),
                                    token.clone(),
                                );
                                self.reporter.error(report);
                                return Err(());
                            }

                            expr = Expr::Set(
                                Rc::new(token),
                                Box::new(expr),
                                Box::new(key),
                                Box::new(self.expr(postfix_precedence, true)?),
                            );
                        } else {
                            expr = Expr::Get(Rc::new(token), Box::new(expr), Box::new(key));
                        }
                    }
                    //TODO<<
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn block(&mut self) -> Result<Stml<'b>, ()> {
        let mut stmls = vec![];
        if !self.check(TokenType::CBrace) {
            while !self.at_end() && !self.check(TokenType::CBrace) {
                stmls.push(self.stml()?);
            }
        };
        self.consume(TokenType::CBrace, "توقعت '}' لنهاية المجموعة")?;
        Ok(Stml::Block(stmls))
    }

    fn return_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Return(Box::new(self.expr(9, true)?)))
    }

    fn throw_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Throw(
            Rc::new(match &self.previous {
                Some(token) => token.clone(),
                None => unreachable!(),
            }),
            Box::new(self.expr(9, true)?),
        ))
    }

    fn params(&mut self) -> Result<Vec<Rc<Token<'b>>>, ()> {
        let mut params = vec![];

        if self.check(TokenType::Identifier) {
            self.consume(TokenType::Identifier, "توقعت اسم معامل")?;
            params.push(Rc::new(match &self.previous {
                Some(token) => token.clone(),
                None => unreachable!(),
            }));
        }
        while self.check(TokenType::Comma) {
            self.advance()?;
            if self.check(TokenType::CParen) {
                break;
            }
            self.consume(TokenType::Identifier, "توقعت اسم معامل")?;
            params.push(Rc::new(match &self.previous {
                Some(token) => token.clone(),
                None => unreachable!(),
            }));
        }

        Ok(params)
    }

    fn function_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الدالة")?;
        let name = match &self.previous {
            Some(token) => token.clone(),
            None => unreachable!(),
        };
        self.consume(TokenType::OParen, "توقعت '(' قبل المعاملات")?;
        let params = self.params()?;
        self.consume(TokenType::CParen, "توقعت ')' بعد المعاملات")?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات")?;
        let body = self.block()?;
        Ok(Stml::Function(Rc::new(name), params, Box::new(body)))
    }

    fn expr_stml(&mut self) -> Result<Stml<'b>, ()> {
        let expr = self.expr(9, true)?;

        Ok(Stml::Expr(expr))
    }

    fn while_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط")?;
        let condition = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط")?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط")?;
        let body = self.block()?;
        Ok(Stml::While(Box::new(condition), Box::new(body)))
    }

    fn loop_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'")?;
        let body = self.block()?;
        Ok(Stml::Loop(Box::new(body)))
    }

    fn break_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Break)
    }

    fn continue_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Continue)
    }

    fn try_catch(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'")?;
        let body = self.block()?;
        self.consume(TokenType::Catch, "توقعت 'أمسك'")?;
        self.consume(TokenType::OParen, "توقعت '('")?;
        let name = match &self.previous {
            Some(token) => token.clone(),
            None => unreachable!(),
        };
        self.consume(TokenType::Identifier, "توقعت اسم المعامل الذي سيحمل الخطأ")?;
        self.consume(TokenType::CParen, "توقعت ')'")?;
        self.consume(TokenType::OBrace, "توقعت '{'")?;
        let catch_body = self.block()?;
        Ok(Stml::TryCatch(
            Box::new(body),
            Rc::new(name),
            Box::new(catch_body),
        ))
    }

    fn if_else_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط")?;
        let condition = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط")?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط")?;
        let if_body = self.block()?;
        let else_body = if self.check(TokenType::Else) {
            self.advance()?;
            self.consume(TokenType::OBrace, "توقعت '{' إلا")?;
            Some(Box::new(self.block()?))
        } else {
            None
        };

        Ok(Stml::IfElse(
            Box::new(condition),
            Box::new(if_body),
            else_body,
        ))
    }

    fn stml(&mut self) -> Result<Stml<'b>, ()> {
        if self.check(TokenType::Function) {
            self.advance()?;
            self.function_stml()
        } else if self.check(TokenType::While) {
            self.advance()?;
            self.while_stml()
        } else if self.check(TokenType::Loop) {
            self.advance()?;
            self.loop_stml()
        } else if self.check(TokenType::If) {
            self.advance()?;
            self.if_else_stml()
        } else if self.check(TokenType::Try) {
            self.advance()?;
            self.try_catch()
        } else if self.check(TokenType::OBrace) {
            self.advance()?;
            self.block()
        } else if self.check(TokenType::Break) {
            self.advance()?;
            self.break_stml()
        } else if self.check(TokenType::Continue) {
            self.advance()?;
            self.continue_stml()
        } else if self.check(TokenType::Return) {
            self.advance()?;
            self.return_stml()
        } else if self.check(TokenType::Throw) {
            self.advance()?;
            self.throw_stml()
        } else {
            self.expr_stml()
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr<'b>, ()> {
        self.advance()?;
        self.expr(9, true)
    }

    pub fn parse(&mut self) -> Result<Vec<Stml<'b>>, ()> {
        let mut stmls = vec![];
        self.advance()?;
        while !self.at_end() {
            stmls.push(self.stml()?);
        }
        Ok(stmls)
    }
}
