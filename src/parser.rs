use super::ast::{Expr, Literal, Stml};
use super::operators::{Associativity, OPERATORS};
use super::reporter::{Phase, Report, Reporter};
use super::token::{Token, TokenType, INVALID_TYPES, STATEMENT_BOUNDRIES};
use super::tokenizer::Tokenizer;
use std::rc::Rc;

pub struct Parser<'a, 'b, 'c> {
    current: Token<'b>,
    previous: Option<Token<'b>>,
    tokenizer: &'a mut Tokenizer<'b>,
    reporter: &'c mut dyn Reporter<'b>,
    had_error: bool,
}

impl<'a, 'b, 'c> Parser<'a, 'b, 'c> {
    pub fn new(tokenizer: &'a mut Tokenizer<'b>, reporter: &'c mut dyn Reporter<'b>) -> Self {
        Self {
            current: tokenizer.next_token(reporter),
            previous: None,
            tokenizer,
            reporter,
            had_error: false,
        }
    }

    fn check_previous(&self) -> Result<(), ()> {
        match &self.previous {
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
        loop {
            if self.current.typ == TokenType::NewLine || self.current.typ == TokenType::Comment {
                self.current = self.tokenizer.next_token(self.reporter);
                continue;
            }
            if self.current.typ == TokenType::EOF {
                break;
            }

            self.previous = Some(self.current.clone());
            self.check_previous()?;
            self.current = self.tokenizer.next_token(self.reporter);
            break;
        }

        Ok(())
    }

    fn next(&mut self) -> Result<Token<'b>, ()> {
        self.advance()?;
        Ok(self.previous.as_ref().unwrap().clone())
    }

    fn consume(
        &mut self,
        typ: TokenType,
        msg: &'static str,
        ignore_newlines: bool,
    ) -> Result<(), ()> {
        if self.check(typ, ignore_newlines) {
            self.advance()?;
            Ok(())
        } else {
            let report = Report::new(Phase::Parsing, msg.to_string(), self.current.clone());
            self.reporter.error(report);
            Err(())
        }
    }

    fn peek(&mut self, ignore_newlines: bool) -> Token<'b> {
        loop {
            if self.current.typ == TokenType::Comment
                || ignore_newlines && self.current.typ == TokenType::NewLine
            {
                self.current = self.tokenizer.next_token(self.reporter);
                continue;
            }
            break;
        }

        self.current.clone()
    }

    fn check(&mut self, typ: TokenType, ignore_newlines: bool) -> bool {
        if self.peek(ignore_newlines).typ == typ {
            return true;
        }

        false
    }

    fn at_end(&mut self) -> bool {
        self.check(TokenType::EOF, true)
    }

    fn exprs(&mut self) -> Result<Vec<Expr<'b>>, ()> {
        let mut items = vec![self.expr(9, true)?];
        while self.check(TokenType::Comma, true) {
            self.advance()?;
            if self.check(TokenType::CBracket, true) || self.check(TokenType::CParen, true) {
                break;
            }
            items.push(self.expr(9, true)?);
        }
        Ok(items)
    }

    fn list(&mut self) -> Result<Expr<'b>, ()> {
        let items = if self.check(TokenType::CBracket, true) {
            vec![]
        } else {
            self.exprs()?
        };

        self.consume(TokenType::CBracket, "توقعت ']' بعد القائمة", true)?;

        Ok(Expr::Literal(Literal::List(items)))
    }

    fn property(&mut self) -> Result<(Rc<Token<'b>>, Expr<'b>), ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الخاصية", true)?;
        let key = self.previous.as_ref().unwrap().clone();
        self.consume(TokenType::Colon, "توقعت ':' بعد الاسم", true)?;
        Ok((Rc::new(key), self.expr(9, true)?))
    }

    fn object(&mut self) -> Result<Expr<'b>, ()> {
        let mut items;
        if self.check(TokenType::CBrace, true) {
            items = vec![]
        } else {
            items = vec![self.property()?];
            while self.check(TokenType::Comma, true) {
                self.advance()?;
                if self.check(TokenType::CBrace, true) {
                    break;
                }
                items.push(self.property()?);
            }
        };

        self.consume(TokenType::CBrace, "توقعت '}' بعد القائمة", true)?;

        Ok(Expr::Literal(Literal::Object(items)))
    }

    fn literal(&mut self) -> Result<Expr<'b>, ()> {
        let token = self.previous.as_ref().unwrap().clone();

        match token.typ {
            TokenType::Identifier => Ok(Expr::Variable(Rc::new(token))),
            TokenType::Number => Ok(Expr::Literal(Literal::Number(Rc::new(token)))),
            TokenType::String => Ok(Expr::Literal(Literal::String(Rc::new(token)))),
            TokenType::True | TokenType::False => Ok(Expr::Literal(Literal::Bool(Rc::new(token)))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil(Rc::new(token)))),
            TokenType::OBracket => self.list(),
            TokenType::OBrace => self.object(),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self) -> Result<Expr<'b>, ()> {
        let token = self.previous.as_ref().unwrap().clone();

        let row: usize = token.typ.into();
        let prefix_precedence = OPERATORS[row].0.unwrap();
        let right = self.expr(prefix_precedence, false)?;
        Ok(Expr::Unary(Rc::new(token), Box::new(right)))
    }

    fn group(&mut self) -> Result<Expr<'b>, ()> {
        let expr = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' لإغلاق المجموعة", true)?;

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

        while !self.check(TokenType::NewLine, false) && !self.at_end() {
            token = self.peek(true);

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
                        let args = if self.check(TokenType::CParen, true) {
                            vec![]
                        } else {
                            self.exprs()?
                        };
                        self.consume(TokenType::CParen, "توقعت ')' بعد القائمة", true)?;

                        expr = Expr::Call(Rc::new(token), Box::new(expr), args);
                    }
                    //TODO>> abstract
                    TokenType::Period => {
                        self.consume(TokenType::Identifier, "توقعت اسم الخاصية", true)?;
                        let key = Expr::Variable(Rc::new(self.previous.as_ref().unwrap().clone()));

                        if self.check(TokenType::Equal, true) {
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
                        self.consume(TokenType::CBracket, "توقعت ']' بعد العبارة", true)?;
                        if self.check(TokenType::Equal, true) {
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
        if !self.check(TokenType::CBrace, true) {
            while !self.at_end() && !self.check(TokenType::CBrace, true) {
                stmls.push(self.stml()?);
            }
        };
        self.consume(TokenType::CBrace, "توقعت '}' لنهاية المجموعة", true)?;
        Ok(Stml::Block(stmls))
    }

    fn return_stml(&mut self) -> Result<Stml<'b>, ()> {
        if self.check(TokenType::NewLine, false) {
            return Ok(Stml::Return(None));
        }

        Ok(Stml::Return(Some(Box::new(self.expr(9, true)?))))
    }

    fn throw_stml(&mut self) -> Result<Stml<'b>, ()> {
        let token = self.previous.as_ref().unwrap().clone();

        if self.check(TokenType::NewLine, false) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }

        Ok(Stml::Throw(
            Rc::new(token),
            Some(Box::new(self.expr(9, true)?)),
        ))
    }

    fn params(&mut self) -> Result<Vec<Rc<Token<'b>>>, ()> {
        let mut params = vec![];

        if self.check(TokenType::Identifier, true) {
            self.consume(TokenType::Identifier, "توقعت اسم معامل", true)?;
            params.push(Rc::new(self.previous.as_ref().unwrap().clone()));
        }
        while self.check(TokenType::Comma, true) {
            self.advance()?;
            if self.check(TokenType::CParen, true) {
                break;
            }
            self.consume(TokenType::Identifier, "توقعت اسم معامل", true)?;
            params.push(Rc::new(self.previous.as_ref().unwrap().clone()));
        }

        Ok(params)
    }

    fn function_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الدالة", true)?;
        let name = self.previous.as_ref().unwrap().clone();
        self.consume(TokenType::OParen, "توقعت '(' قبل المعاملات", true)?;
        let params = self.params()?;
        self.consume(TokenType::CParen, "توقعت ')' بعد المعاملات", true)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات", true)?;
        let body = self.block()?;
        Ok(Stml::Function(Rc::new(name), params, Box::new(body)))
    }

    fn expr_stml(&mut self) -> Result<Stml<'b>, ()> {
        let expr = self.expr(9, true)?;

        Ok(Stml::Expr(expr))
    }

    fn while_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط", true)?;
        let condition = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط", true)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط", true)?;
        let body = self.block()?;
        Ok(Stml::While(Box::new(condition), Box::new(body)))
    }

    fn loop_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'", true)?;
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
        self.consume(TokenType::OBrace, "توقعت '{'", true)?;
        let body = self.block()?;
        self.consume(TokenType::Catch, "توقعت 'أمسك'", true)?;
        self.consume(TokenType::OParen, "توقعت '('", true)?;
        let name = self.previous.as_ref().unwrap().clone();
        self.consume(
            TokenType::Identifier,
            "توقعت اسم المعامل الذي سيحمل الخطأ",
            true,
        )?;
        self.consume(TokenType::CParen, "توقعت ')'", true)?;
        self.consume(TokenType::OBrace, "توقعت '{'", true)?;
        let catch_body = self.block()?;
        Ok(Stml::TryCatch(
            Box::new(body),
            Rc::new(name),
            Box::new(catch_body),
        ))
    }

    fn if_else_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط", true)?;
        let condition = self.expr(9, true)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط", true)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط", true)?;
        let if_body = self.block()?;
        let else_body = if self.check(TokenType::Else, true) {
            self.advance()?;
            self.consume(TokenType::OBrace, "توقعت '{' إلا", true)?;
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
        if self.check(TokenType::Function, true) {
            self.advance()?;
            self.function_stml()
        } else if self.check(TokenType::While, true) {
            self.advance()?;
            self.while_stml()
        } else if self.check(TokenType::Loop, true) {
            self.advance()?;
            self.loop_stml()
        } else if self.check(TokenType::If, true) {
            self.advance()?;
            self.if_else_stml()
        } else if self.check(TokenType::Try, true) {
            self.advance()?;
            self.try_catch()
        } else if self.check(TokenType::OBrace, true) {
            self.advance()?;
            self.block()
        } else if self.check(TokenType::Break, true) {
            self.advance()?;
            self.break_stml()
        } else if self.check(TokenType::Continue, true) {
            self.advance()?;
            self.continue_stml()
        } else if self.check(TokenType::Return, true) {
            self.advance()?;
            self.return_stml()
        } else if self.check(TokenType::Throw, true) {
            self.advance()?;
            self.throw_stml()
        } else {
            self.expr_stml()
        }
    }

    fn sync(&mut self) {
        while !self.check(TokenType::EOF, true) {
            if STATEMENT_BOUNDRIES.contains(&self.peek(true).typ) {
                break;
            }
            self.advance().ok();
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr<'b>, ()> {
        self.expr(9, true)
    }

    pub fn parse(&mut self) -> Result<Vec<Stml<'b>>, ()> {
        let mut stmls = vec![];
        while !self.at_end() {
            match self.stml() {
                Ok(stml) => stmls.push(stml),
                Err(_) => {
                    self.had_error = true;
                    self.sync();
                }
            }
        }
        if self.had_error {
            Err(())
        } else {
            Ok(stmls)
        }
    }
}
