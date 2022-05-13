use super::ast::{Expr, Literal, Stml};
use super::operators::{Associativity, OPERATORS};
use super::reporter::{Phase, Report, Reporter};
use super::token::{Token, TokenType, BOUNDARIES, INVALID_TYPES};
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

    fn error_at(&mut self, token: &Token<'b>, msg: &str) {
        let report = Report::new(Phase::Parsing, msg.to_string(), Rc::new(token.clone()));
        self.reporter.error(report);
        self.had_error = true;
    }

    fn warning_at(&mut self, token: &Token<'b>, msg: &str) {
        let report = Report::new(Phase::Parsing, msg.to_string(), Rc::new(token.clone()));
        self.reporter.warning(report);
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

    fn consume(&mut self, typ: TokenType, msg: &'static str) -> Result<(), ()> {
        if self.check(typ) {
            self.advance()?;
            Ok(())
        } else {
            let token = self.current.clone();
            self.error_at(&token, msg);
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

    fn check(&mut self, typ: TokenType) -> bool {
        let ignore_newlines = typ != TokenType::NewLine;

        if self.peek(ignore_newlines).typ == typ {
            return true;
        }

        false
    }

    fn at_end(&mut self) -> bool {
        self.check(TokenType::EOF)
    }

    fn exprs(&mut self) -> Result<Vec<Expr<'b>>, ()> {
        let mut items = vec![self.parse_expr()?];
        while self.check(TokenType::Comma) {
            self.advance()?;
            if self.check(TokenType::CBracket) || self.check(TokenType::CParen) {
                break;
            }
            items.push(self.parse_expr()?);
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
        let key = self.previous.as_ref().unwrap().clone();
        self.consume(TokenType::Colon, "توقعت ':' بعد الاسم")?;
        Ok((Rc::new(key), self.parse_expr()?))
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
        let expr = self.parse_expr()?;
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
            TokenType::OParen => {
                can_assign = false;
                self.group()?
            }
            _ => {
                self.error_at(&token, "توقعت عبارة");
                return Err(());
            }
        };

        while !self.check(TokenType::NewLine) && !self.at_end() {
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
                    self.error_at(&token, "الجانب الأيمن غير صحيح");
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
                        let key = Expr::Variable(Rc::new(self.previous.as_ref().unwrap().clone()));

                        if self.check(TokenType::Equal) {
                            token = self.next()?;
                            if !can_assign {
                                self.error_at(&token, "الجانب الأيمن غير صحيح");
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
                        let key = self.parse_expr()?;
                        self.consume(TokenType::CBracket, "توقعت ']' بعد العبارة")?;
                        if self.check(TokenType::Equal) {
                            let row: usize = self.peek(true).typ.into();
                            let infix_precedence = OPERATORS[row].1.unwrap();
                            token = self.next()?;
                            if !can_assign {
                                self.error_at(&token, "الجانب الأيمن غير صحيح");
                                return Err(());
                            }

                            expr = Expr::Set(
                                Rc::new(token),
                                Box::new(expr),
                                Box::new(key),
                                Box::new(self.expr(infix_precedence, true)?),
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
                stmls.push(self.decl()?);
            }
        };
        self.consume(TokenType::CBrace, "توقعت '}' لنهاية المجموعة")?;
        Ok(Stml::Block(stmls))
    }

    fn return_stml(&mut self) -> Result<Stml<'b>, ()> {
        let token = self.previous.as_ref().unwrap().clone();

        if self.check(TokenType::NewLine) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }

        Ok(Stml::Return(Rc::new(token), Some(self.parse_expr()?)))
    }

    fn throw_stml(&mut self) -> Result<Stml<'b>, ()> {
        let token = self.previous.as_ref().unwrap().clone();

        if self.check(TokenType::NewLine) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }

        Ok(Stml::Throw(Rc::new(token), Some(self.parse_expr()?)))
    }

    fn params(&mut self) -> Result<Vec<Rc<Token<'b>>>, ()> {
        let mut params = vec![];

        if self.check(TokenType::Identifier) {
            self.consume(TokenType::Identifier, "توقعت اسم معامل")?;
            params.push(Rc::new(self.previous.as_ref().unwrap().clone()));
        }
        while self.check(TokenType::Comma) {
            self.advance()?;
            if self.check(TokenType::CParen) {
                break;
            }
            self.consume(TokenType::Identifier, "توقعت اسم معامل")?;
            params.push(Rc::new(self.previous.as_ref().unwrap().clone()));
        }

        Ok(params)
    }

    fn function_decl(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الدالة")?;
        let name = self.previous.as_ref().unwrap().clone();
        self.consume(TokenType::OParen, "توقعت '(' قبل المعاملات")?;
        let params = self.params()?;
        self.consume(TokenType::CParen, "توقعت ')' بعد المعاملات")?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات")?;
        let body = self.block()?;
        Ok(Stml::FunctionDecl(Rc::new(name), params, Box::new(body)))
    }

    fn expr_stml(&mut self) -> Result<Stml<'b>, ()> {
        let expr = self.parse_expr()?;

        Ok(Stml::Expr(expr))
    }

    fn while_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط")?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط")?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط")?;
        let body = self.block()?;
        Ok(Stml::While(condition, Box::new(body)))
    }

    fn loop_stml(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'")?;
        let body = self.block()?;
        Ok(Stml::Loop(Box::new(body)))
    }

    fn break_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Break(Rc::new(
            self.previous.as_ref().unwrap().clone(),
        )))
    }

    fn continue_stml(&mut self) -> Result<Stml<'b>, ()> {
        Ok(Stml::Continue(Rc::new(
            self.previous.as_ref().unwrap().clone(),
        )))
    }

    fn try_catch(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'")?;
        let body = self.block()?;
        self.consume(TokenType::Catch, "توقعت 'أمسك'")?;
        self.consume(TokenType::OParen, "توقعت '('")?;
        let name = self.previous.as_ref().unwrap().clone();
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
        let condition = self.parse_expr()?;
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

        Ok(Stml::IfElse(condition, Box::new(if_body), else_body))
    }

    fn var_decl(&mut self) -> Result<Stml<'b>, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم المتغير")?;
        let name = self.previous.as_ref().unwrap().clone();
        let initializer = if self.check(TokenType::Equal) {
            self.advance()?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Stml::VarDecl(Rc::new(name), initializer))
    }

    fn stml(&mut self) -> Result<Stml<'b>, ()> {
        if self.check(TokenType::While) {
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

    fn decl(&mut self) -> Result<Stml<'b>, ()> {
        if self.check(TokenType::Function) {
            self.advance()?;
            self.function_decl()
        } else if self.check(TokenType::Var) {
            self.advance()?;
            self.var_decl()
        } else {
            self.stml()
        }
    }

    fn sync(&mut self) {
        while !self.check(TokenType::EOF) {
            if BOUNDARIES.contains(&self.peek(true).typ) {
                break;
            }
            self.advance().ok();
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr<'b>, ()> {
        self.expr(9, true)
    }

    pub fn parse(&mut self) -> Result<Vec<Stml<'b>>, ()> {
        let mut decls = vec![];
        while !self.at_end() {
            match self.decl() {
                Ok(decl) => decls.push(decl),
                Err(_) => {
                    self.sync();
                }
            }
        }
        if self.had_error {
            Err(())
        } else {
            Ok(decls)
        }
    }
}
