use super::{
    ast::{Expr, Literal, Stml},
    operators::{Associativity, OPERATORS},
    reporter::{Phase, Report, Reporter},
    token::{Token, TokenType, BINARY_SET, BOUNDARIES},
    tokenizer::Tokenizer,
};
use std::{path::PathBuf, rc::Rc};

pub struct Parser {
    tokenizer: Tokenizer,
    current: Token,
    previous: Option<Token>,
    had_error: bool,
}

impl Parser {
    pub fn new(source: String, path: Option<PathBuf>) -> Self {
        let mut tokenizer = Tokenizer::new(source, path);
        let current = tokenizer.next_token();

        Self {
            tokenizer,
            current,
            previous: None,
            had_error: false,
        }
    }

    fn error_at(&mut self, phase: Phase, token: &Token, msg: &str, reporter: &mut dyn Reporter) {
        let report = Report::new(phase, msg.to_string(), Rc::new(token.clone()));
        reporter.error(report);
        self.had_error = true;
    }

    // fn warning_at(&mut self, token: &Token, msg: &str) {
    //     let report = Report::new(Phase::Parsing, msg.to_string(), Rc::new(token.clone()));
    //     self.reporter.warning(report);
    // }

    fn advance(&mut self, reporter: &mut dyn Reporter) -> Result<(), ()> {
        loop {
            if self.current.typ == TokenType::NewLine || self.current.typ == TokenType::Comment {
                self.current = self.tokenizer.next_token();
                continue;
            }

            self.previous = Some(self.current.clone());

            if self.current.typ == TokenType::EOF {
                break;
            }

            self.current = self.tokenizer.next_token();

            if let Some(token) = self.previous.clone() {
                match token.typ {
                    TokenType::Unknown => {
                        self.error_at(Phase::Tokenizing, &token, "رمز غير متوقع", reporter);
                        return Err(());
                    }
                    TokenType::UnTermedString => {
                        self.error_at(Phase::Tokenizing, &token, "نص غير مغلق", reporter);
                        return Err(());
                    }
                    TokenType::InvalidNumber => {
                        self.error_at(Phase::Tokenizing, &token, "رقم خاطئ", reporter);
                        return Err(());
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn next(&mut self, reporter: &mut dyn Reporter) -> Result<Token, ()> {
        self.advance(reporter)?;
        Ok(self.clone_previous())
    }

    fn consume(
        &mut self,
        typ: TokenType,
        msg: &'static str,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if self.check(typ) {
            self.advance(reporter)?;
            Ok(())
        } else {
            let token = self.current.clone();
            self.error_at(Phase::Parsing, &token, msg, reporter);
            Err(())
        }
    }

    fn peek(&mut self, ignore_newlines: bool) -> Token {
        loop {
            if self.current.typ == TokenType::Comment
                || ignore_newlines && self.current.typ == TokenType::NewLine
            {
                self.current = self.tokenizer.next_token();
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

    fn clone_previous(&self) -> Token {
        self.previous.as_ref().unwrap().clone()
    }

    fn exprs(&mut self, reporter: &mut dyn Reporter) -> Result<Vec<Expr>, ()> {
        let mut items = vec![self.parse_expr(reporter)?];
        while self.check(TokenType::Comma) {
            self.advance(reporter)?;
            if self.check(TokenType::CBracket) || self.check(TokenType::CParen) {
                break;
            }
            items.push(self.parse_expr(reporter)?);
        }
        Ok(items)
    }

    fn list(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        let items = if self.check(TokenType::CBracket) {
            vec![]
        } else {
            self.exprs(reporter)?
        };

        self.consume(TokenType::CBracket, "توقعت ']' بعد القائمة", reporter)?;

        Ok(Expr::Literal(Literal::List(items)))
    }

    fn property(&mut self, reporter: &mut dyn Reporter) -> Result<(Rc<Token>, Expr), ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الخاصية", reporter)?;
        let key = self.clone_previous();
        self.consume(TokenType::Colon, "توقعت ':' بعد الاسم", reporter)?;
        Ok((Rc::new(key), self.parse_expr(reporter)?))
    }
    fn object(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        let mut items;
        if self.check(TokenType::CBrace) {
            items = vec![]
        } else {
            items = vec![self.property(reporter)?];
            while self.check(TokenType::Comma) {
                self.advance(reporter)?;
                if self.check(TokenType::CBrace) {
                    break;
                }
                items.push(self.property(reporter)?);
            }
        };

        self.consume(TokenType::CBrace, "توقعت '}' بعد القائمة", reporter)?;

        Ok(Expr::Literal(Literal::Object(items)))
    }

    fn literal(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        let token = self.clone_previous();

        match token.typ {
            TokenType::Identifier => Ok(Expr::Variable(Rc::new(token))),
            TokenType::Number => Ok(Expr::Literal(Literal::Number(Rc::new(token)))),
            TokenType::String => Ok(Expr::Literal(Literal::String(Rc::new(token)))),
            TokenType::True | TokenType::False => Ok(Expr::Literal(Literal::Bool(Rc::new(token)))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil(Rc::new(token)))),
            TokenType::OBracket => self.list(reporter),
            TokenType::OBrace => self.object(reporter),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        let token = self.clone_previous();

        let row: usize = token.typ.into();
        let prefix_precedence = OPERATORS[row].0.unwrap();
        let right = self.expr(prefix_precedence, false, reporter)?;
        Ok(Expr::Unary(Rc::new(token), Box::new(right)))
    }

    fn group(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        let expr = self.parse_expr(reporter)?;
        self.consume(TokenType::CParen, "توقعت ')' لإغلاق المجموعة", reporter)?;
        return Ok(expr);
    }

    fn lambda(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        if self.clone_previous().typ == TokenType::Or {
            self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات", reporter)?;
            let body = self.block(reporter)?;
            Ok(Expr::Lambda(Vec::new(), Box::new(body)))
        } else {
            let params = self.params(reporter)?;
            self.consume(TokenType::Pipe, "توقعت '|'", reporter)?;
            self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات", reporter)?;
            let body = self.block(reporter)?;
            Ok(Expr::Lambda(params, Box::new(body)))
        }
    }

    /// Parses any expression with a binding power more than or equal to `min_bp`.
    fn expr(
        &mut self,
        min_precedence: u8,
        mut can_assign: bool,
        reporter: &mut dyn Reporter,
    ) -> Result<Expr, ()> {
        let mut token = self.next(reporter)?;
        let mut expr;

        expr = match token.typ {
            TokenType::Identifier
            | TokenType::Number
            | TokenType::String
            | TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::OBracket
            | TokenType::OBrace => self.literal(reporter)?,
            TokenType::Minus | TokenType::Bang => self.unary(reporter)?,
            TokenType::OParen => {
                can_assign = false;
                self.group(reporter)?
            }
            TokenType::Pipe | TokenType::Or => {
                can_assign = false;
                self.lambda(reporter)?
            }
            _ => {
                self.error_at(Phase::Parsing, &token, "توقعت عبارة", reporter);
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

                self.advance(reporter)?;

                if token.typ == TokenType::Equal && !can_assign {
                    self.error_at(Phase::Parsing, &token, "الجانب الأيمن غير صحيح", reporter);
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
                        reporter,
                    )?),
                );
            } else if let Some(postfix_precedence) = OPERATORS[row as usize].2 {
                if min_precedence < postfix_precedence {
                    break;
                }
                self.advance(reporter)?;

                match token.typ {
                    TokenType::OParen => {
                        let args = if self.check(TokenType::CParen) {
                            vec![]
                        } else {
                            self.exprs(reporter)?
                        };
                        self.consume(TokenType::CParen, "توقعت ')' بعد القائمة", reporter)?;

                        expr = Expr::Call(Rc::new(token), Box::new(expr), args);
                    }
                    //TODO>> abstract
                    TokenType::Period => {
                        self.consume(TokenType::Identifier, "توقعت اسم الخاصية", reporter)?;
                        let key = Expr::Literal(Literal::String(Rc::new(self.clone_previous())));

                        if BINARY_SET.contains(&self.peek(true).typ) {
                            token = self.next(reporter)?;
                            if !can_assign {
                                self.error_at(
                                    Phase::Parsing,
                                    &token,
                                    "الجانب الأيمن غير صحيح",
                                    reporter,
                                );
                            }

                            expr = Expr::Set(
                                Rc::new(token),
                                Box::new(expr),
                                Box::new(key),
                                Box::new(self.expr(postfix_precedence, true, reporter)?),
                            );
                        } else {
                            expr = Expr::Get(Rc::new(token), Box::new(expr), Box::new(key));
                        }
                    }
                    TokenType::OBracket => {
                        let key = self.parse_expr(reporter)?;
                        self.consume(TokenType::CBracket, "توقعت ']' بعد العبارة", reporter)?;
                        if BINARY_SET.contains(&self.peek(true).typ) {
                            let row: usize = self.peek(true).typ.into();
                            let infix_precedence = OPERATORS[row].1.unwrap();
                            token = self.next(reporter)?;
                            if !can_assign {
                                self.error_at(
                                    Phase::Parsing,
                                    &token,
                                    "الجانب الأيمن غير صحيح",
                                    reporter,
                                );
                            }

                            expr = Expr::Set(
                                Rc::new(token),
                                Box::new(expr),
                                Box::new(key),
                                Box::new(self.expr(infix_precedence, true, reporter)?),
                            );
                        } else {
                            expr = Expr::Get(Rc::new(token), Box::new(expr), Box::new(key));
                        }
                    }
                    //<<
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn block(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        let mut stmls = vec![];
        if !self.check(TokenType::CBrace) {
            while !self.at_end() && !self.check(TokenType::CBrace) {
                stmls.push(self.decl(reporter)?);
            }
        };
        self.consume(TokenType::CBrace, "توقعت '}' لنهاية المجموعة", reporter)?;
        Ok(Stml::Block(stmls))
    }

    fn return_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        let token = self.clone_previous();

        if self.check(TokenType::NewLine) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }

        Ok(Stml::Return(
            Rc::new(token),
            Some(self.parse_expr(reporter)?),
        ))
    }

    fn throw_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        let token = self.clone_previous();

        if self.check(TokenType::NewLine) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }

        Ok(Stml::Throw(
            Rc::new(token),
            Some(self.parse_expr(reporter)?),
        ))
    }

    fn params(&mut self, reporter: &mut dyn Reporter) -> Result<Vec<Rc<Token>>, ()> {
        let mut params = vec![];

        if self.check(TokenType::Identifier) {
            self.consume(TokenType::Identifier, "توقعت اسم معامل", reporter)?;
            params.push(Rc::new(self.clone_previous()));
        }

        while self.check(TokenType::Comma) {
            self.advance(reporter)?;
            if self.check(TokenType::CParen) || self.check(TokenType::Pipe) {
                break;
            }
            self.consume(TokenType::Identifier, "توقعت اسم معامل", reporter)?;
            params.push(Rc::new(self.clone_previous()));
        }

        Ok(params)
    }

    fn function_decl(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم الدالة", reporter)?;
        let name = self.clone_previous();
        self.consume(TokenType::OParen, "توقعت '(' قبل المعاملات", reporter)?;
        let params = self.params(reporter)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد المعاملات", reporter)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد المعاملات", reporter)?;
        let body = self.block(reporter)?;
        Ok(Stml::FunctionDecl(Rc::new(name), params, Box::new(body)))
    }

    fn expr_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        let expr = self.parse_expr(reporter)?;

        Ok(Stml::Expr(expr))
    }

    fn while_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط", reporter)?;
        let condition = self.parse_expr(reporter)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط", reporter)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط", reporter)?;
        let body = self.block(reporter)?;
        Ok(Stml::While(condition, Box::new(body)))
    }

    fn loop_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'", reporter)?;
        let body = self.block(reporter)?;
        Ok(Stml::Loop(Box::new(body)))
    }

    fn break_stml(&mut self) -> Result<Stml, ()> {
        Ok(Stml::Break(Rc::new(self.clone_previous())))
    }

    fn continue_stml(&mut self) -> Result<Stml, ()> {
        Ok(Stml::Continue(Rc::new(self.clone_previous())))
    }

    fn try_catch(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::OBrace, "توقعت '{'", reporter)?;
        let body = self.block(reporter)?;
        self.consume(TokenType::Catch, "توقعت 'أمسك'", reporter)?;
        self.consume(TokenType::OParen, "توقعت '('", reporter)?;
        self.consume(
            TokenType::Identifier,
            "توقعت اسم المعامل الذي سيحمل الخطأ",
            reporter,
        )?;
        let name = self.clone_previous();
        self.consume(TokenType::CParen, "توقعت ')'", reporter)?;
        self.consume(TokenType::OBrace, "توقعت '{'", reporter)?;
        let catch_body = self.block(reporter)?;
        Ok(Stml::TryCatch(
            Box::new(body),
            Rc::new(name),
            Box::new(catch_body),
        ))
    }

    fn if_else_stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::OParen, "توقعت '(' قبل الشرط", reporter)?;
        let condition = self.parse_expr(reporter)?;
        self.consume(TokenType::CParen, "توقعت ')' بعد الشرط", reporter)?;
        self.consume(TokenType::OBrace, "توقعت '{' بعد الشرط", reporter)?;
        let if_body = self.block(reporter)?;

        let mut elseifs = Vec::new();
        while self.check(TokenType::ElseIf) {
            self.advance(reporter)?;
            self.consume(TokenType::OParen, "توقعت '(' قبل الشرط", reporter)?;
            let condition = self.parse_expr(reporter)?;
            self.consume(TokenType::CParen, "توقعت ')' بعد الشرط", reporter)?;

            self.consume(TokenType::OBrace, "توقعت '{' إلا", reporter)?;
            let body = self.block(reporter)?;

            elseifs.push((condition, body));
        }

        let else_body = if self.check(TokenType::Else) {
            self.advance(reporter)?;
            self.consume(TokenType::OBrace, "توقعت '{' إلا", reporter)?;
            Some(Box::new(self.block(reporter)?))
        } else {
            None
        };

        Ok(Stml::IfElse(
            condition,
            Box::new(if_body),
            elseifs,
            else_body,
        ))
    }

    fn var_decl(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم المتغير", reporter)?;
        let name = self.clone_previous();
        let initializer = if self.check(TokenType::Equal) {
            self.advance(reporter)?;
            Some(self.parse_expr(reporter)?)
        } else {
            None
        };
        Ok(Stml::VarDecl(Rc::new(name), initializer))
    }

    fn stml(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        if self.check(TokenType::While) {
            self.advance(reporter)?;
            self.while_stml(reporter)
        } else if self.check(TokenType::Loop) {
            self.advance(reporter)?;
            self.loop_stml(reporter)
        } else if self.check(TokenType::If) {
            self.advance(reporter)?;
            self.if_else_stml(reporter)
        } else if self.check(TokenType::Try) {
            self.advance(reporter)?;
            self.try_catch(reporter)
        } else if self.check(TokenType::OBrace) {
            self.advance(reporter)?;
            self.block(reporter)
        } else if self.check(TokenType::Break) {
            self.advance(reporter)?;
            self.break_stml()
        } else if self.check(TokenType::Continue) {
            self.advance(reporter)?;
            self.continue_stml()
        } else if self.check(TokenType::Return) {
            self.advance(reporter)?;
            self.return_stml(reporter)
        } else if self.check(TokenType::Throw) {
            self.advance(reporter)?;
            self.throw_stml(reporter)
        } else {
            self.expr_stml(reporter)
        }
    }

    fn imported_decl(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        self.consume(TokenType::Identifier, "توقعت اسم المتغير", reporter)?;
        let name = self.clone_previous();
        self.consume(TokenType::From, "توقعت 'من'", reporter)?;
        self.consume(TokenType::String, "توقعت مسار الملف المستورد", reporter)?;
        Ok(Stml::Import(Rc::new(name), Rc::new(self.clone_previous())))
    }

    fn exported_decl(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        let token = self.clone_previous();

        if self.check(TokenType::Function) {
            self.advance(reporter)?;
            Ok(Stml::Export(
                Rc::new(token),
                Box::new(self.function_decl(reporter)?),
            ))
        } else if self.check(TokenType::Var) {
            self.advance(reporter)?;
            Ok(Stml::Export(
                Rc::new(token),
                Box::new(self.var_decl(reporter)?),
            ))
        } else {
            let token = self.current.clone();
            self.error_at(
                Phase::Parsing,
                &token,
                "يمكنك فقط تصدير الدوال والمتغيرات",
                reporter,
            );
            Err(())
        }
    }

    fn decl(&mut self, reporter: &mut dyn Reporter) -> Result<Stml, ()> {
        if self.check(TokenType::Function) {
            self.advance(reporter)?;
            self.function_decl(reporter)
        } else if self.check(TokenType::Var) {
            self.advance(reporter)?;
            self.var_decl(reporter)
        } else if self.check(TokenType::Export) {
            self.advance(reporter)?;
            self.exported_decl(reporter)
        } else if self.check(TokenType::Import) {
            self.advance(reporter)?;
            self.imported_decl(reporter)
        } else {
            self.stml(reporter)
        }
    }

    fn sync(&mut self, reporter: &mut dyn Reporter) {
        while !self.check(TokenType::EOF) {
            if BOUNDARIES.contains(&self.peek(true).typ) {
                break;
            }
            self.advance(reporter).ok();
        }
    }

    pub fn parse_expr(&mut self, reporter: &mut dyn Reporter) -> Result<Expr, ()> {
        self.expr(9, true, reporter)
    }

    pub fn parse(&mut self, reporter: &mut dyn Reporter) -> Result<Vec<Stml>, ()> {
        if cfg!(feature = "debug-ast") {
            println!("---");
            println!("[DEBUG] started parsing");
            println!("---");
        }

        let mut decls = vec![];
        while !self.at_end() {
            match self.decl(reporter) {
                Ok(decl) => decls.push(decl),
                Err(_) => {
                    self.sync(reporter);
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
