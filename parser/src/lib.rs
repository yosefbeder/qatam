pub mod ast;
pub mod error;
mod operators;

use ast::*;
use error::*;
use lexer::token::*;
use operators::*;
use std::rc::Rc;

#[derive(PartialEq, Clone, Copy)]
enum AssignAbility {
    AnyOp,
    OnlyEqual,
    None,
}

pub struct Parser {
    tokens: Vec<Rc<Token>>,
    /// The token at current represents the next token and it should always be a valid one.
    current: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<Rc<Token>>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: vec![],
        }
    }

    fn can_assign(typ: TokenType, assign_ops: AssignAbility) -> bool {
        match typ {
            TokenType::Equal => assign_ops != AssignAbility::None,
            x if BINARY_SET.contains(&x) => assign_ops == AssignAbility::AnyOp,
            _ => false,
        }
    }

    fn err(&mut self, err: Error) {
        self.errors.push(err);
    }

    fn lexical_err(&mut self, token: Rc<Token>) {
        self.err(Error::Lexical(token))
    }

    fn parse_err(&mut self, err: ParseError) {
        self.err(Error::Parse(err))
    }

    /// Skips new lines until it finds a valid or error token.
    fn peek_no_lines(&mut self) -> Result<Rc<Token>, ()> {
        while self.check(&[TokenType::NewLine])? {
            self.advance()?;
        }
        Ok(self.peek())
    }

    fn previous(&self) -> Rc<Token> {
        Rc::clone(&self.tokens[self.current - 1])
    }

    /// Returns the current token without advancing the iterator.
    fn peek(&self) -> Rc<Token> {
        Rc::clone(&self.tokens[self.current])
    }

    /// Checks if `expected` contains the next token type.
    ///
    /// If `expected` contains a new line token it uses `self.peek()`, otherwise it uses `self.peek_no_lines()`.
    fn check(&mut self, expected: &[TokenType]) -> Result<bool, ()> {
        if expected.contains(&TokenType::NewLine) {
            Ok(expected.contains(&self.peek().typ()))
        } else {
            Ok(expected.contains(&self.peek_no_lines()?.typ()))
        }
    }

    fn at_end(&mut self) -> Result<bool, ()> {
        self.check(&[TokenType::EOF])
    }

    /// Checks if `self.tokens[self.current]` is a valid token and fails if it's not.
    fn validate_current(&mut self) -> Result<(), ()> {
        let token = self.peek();
        if ERROR_TOKENS.contains(&token.typ()) {
            self.lexical_err(token);
            Err(())
        } else {
            Ok(())
        }
    }

    /// Advance `self.current`.
    fn advance(&mut self) -> Result<(), ()> {
        self.current += 1;
        self.validate_current()
    }

    fn next(&mut self) -> Result<Rc<Token>, ()> {
        let token = self.peek();
        self.advance()?;
        Ok(token)
    }

    fn consume(&mut self, expected: &[TokenType]) -> Result<Rc<Token>, ()> {
        let token = self.next()?;
        if !expected.contains(&token.typ()) {
            self.parse_err(ParseError::ExpectedInstead(expected.to_owned(), token));
            Err(())
        } else {
            Ok(token)
        }
    }

    fn check_consume(&mut self, expected: &[TokenType]) -> Result<bool, ()> {
        if self.check(expected)? {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expr(&mut self, min_precedence: u8, mut assign_abililty: AssignAbility) -> Result<Expr, ()> {
        let token = self.next()?;
        let mut expr;

        expr = match token.typ() {
            TokenType::Identifier => Expr::Variable(Rc::clone(&token)),
            TokenType::OBracket | TokenType::OBrace => {
                assign_abililty = AssignAbility::OnlyEqual;
                self.literal()?
            }
            TokenType::Number
            | TokenType::String
            | TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::Pipe => {
                assign_abililty = AssignAbility::None;
                self.literal()?
            }
            TokenType::Minus | TokenType::Bang => {
                assign_abililty = AssignAbility::None;
                self.literal()?
            }
            TokenType::OParen => {
                assign_abililty = AssignAbility::None;
                self.literal()?
            }
            TokenType::EOF => return Err(()),
            _ => {
                self.parse_err(ParseError::ExpectedExpr(token));
                return Err(());
            }
        };

        while !self.check(&[TokenType::NewLine])? && !self.at_end()? {
            let op = self.peek();
            let row: usize = op.typ() as usize;
            if let Some(infix_precedence) = OPERATORS[row].1 {
                let associativity = OPERATORS[row].3.unwrap();
                if min_precedence < infix_precedence {
                    break;
                }
                self.advance()?;
                if !BINARY_SET.contains(&op.typ()) {
                    assign_abililty = AssignAbility::None;
                }
                let can_assign = Self::can_assign(op.typ(), assign_abililty);
                if BINARY_SET.contains(&op.typ()) && !can_assign {
                    self.parse_err(ParseError::InvalidRhs(Rc::clone(&op)));
                }
                expr = Expr::Binary(
                    Box::new(expr),
                    op,
                    Box::new(self.expr(
                        match associativity {
                            Associativity::Right => infix_precedence,
                            Associativity::Left => infix_precedence - 1,
                        },
                        if can_assign {
                            AssignAbility::AnyOp
                        } else {
                            AssignAbility::None
                        },
                    )?),
                );
            } else if let Some(postfix_precedence) = OPERATORS[row as usize].2 {
                if min_precedence < postfix_precedence {
                    break;
                }
                self.advance()?;
                match op.typ() {
                    TokenType::OParen => {
                        assign_abililty = AssignAbility::None;
                        expr = Expr::Call(Box::new(expr), op, self.exprs(TokenType::CParen)?);
                    }
                    TokenType::Period | TokenType::OBracket => {
                        match expr {
                            Expr::Call(_, _, _) => {
                                assign_abililty = AssignAbility::AnyOp;
                            }
                            _ => {}
                        }
                        let key = match op.typ() {
                            TokenType::Period => {
                                self.consume(&[TokenType::Identifier])?;
                                Expr::Literal(Literal::String(self.previous()))
                            }
                            TokenType::OBracket => {
                                let tmp = self.parse_expr()?;
                                self.consume(&[TokenType::CBracket])?;
                                tmp
                            }
                            _ => unreachable!(),
                        };

                        expr = Expr::Member(Box::new(expr), op, Box::new(key));
                    }
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn literal(&mut self) -> Result<Expr, ()> {
        let token = self.previous();
        match token.typ() {
            TokenType::Number => Ok(Expr::Literal(Literal::Number(token))),
            TokenType::String => Ok(Expr::Literal(Literal::String(token))),
            TokenType::True | TokenType::False => Ok(Expr::Literal(Literal::Bool(token))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil(token))),
            TokenType::OBracket => Ok(self.list()?.into()),
            TokenType::OBrace => Ok(self.object()?.into()),
            TokenType::Pipe => Ok(self.lambda()?.into()),
            _ => unreachable!(),
        }
    }

    fn exprs(&mut self, closing_token: TokenType) -> Result<Vec<Expr>, ()> {
        let mut exprs = vec![];
        if !self.check(&[closing_token])? {
            exprs.push(self.parse_expr()?);
            while self.check_consume(&[TokenType::Comma])? {
                if self.check(&[closing_token])? {
                    break;
                }
                exprs.push(self.parse_expr()?)
            }
        }
        self.consume(&[closing_token])?;
        Ok(exprs)
    }

    fn list(&mut self) -> Result<Literal, ()> {
        let token = self.previous();
        Ok(Literal::List(token, self.exprs(TokenType::CBracket)?))
    }

    fn prop(&mut self) -> Result<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>), ()> {
        self.consume(&[TokenType::Identifier])?;
        let key = self.previous();
        let mut value = if self.check_consume(&[TokenType::Colon])? {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let default = match value.clone() {
            Some(Expr::Binary(lhs, op, rhs)) if op.typ() == TokenType::Equal => {
                value = Some(*lhs);
                Some((op, *rhs))
            }
            None => {
                if self.check_consume(&[TokenType::Equal])? {
                    Some((self.previous(), self.parse_expr()?))
                } else {
                    None
                }
            }
            _ => None,
        };
        Ok((key, value, default))
    }

    fn props(&mut self) -> Result<Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>, ()> {
        let mut props = vec![];
        if !self.check(&[TokenType::CBrace])? {
            props.push(self.prop()?);
            while self.check_consume(&[TokenType::Comma])? {
                if self.check(&[TokenType::CBrace])? {
                    break;
                }
                props.push(self.prop()?)
            }
        }
        self.consume(&[TokenType::CBrace])?;
        Ok(props)
    }

    fn object(&mut self) -> Result<Literal, ()> {
        let token = self.previous();
        Ok(Literal::Object(token, self.props()?))
    }

    fn lambda(&mut self) -> Result<Literal, ()> {
        todo!()
    }

    fn parse_expr(&mut self) -> Result<Expr, ()> {
        self.expr(9, AssignAbility::AnyOp)
    }

    fn definable(&mut self) -> Result<Expr, ()> {
        todo!()
    }

    fn import_stml(&mut self) -> Result<Stml, ()> {
        let token = self.previous();
        let definable = self.definable()?;
        let from_token = self.consume(&[TokenType::From])?;
        let path = self.consume(&[TokenType::String])?;
        Ok(Stml::Import(token, definable, from_token, path))
    }

    fn expr_stml(&mut self) -> Result<Stml, ()> {
        Ok(Stml::Expr(self.parse_expr()?))
    }

    fn stml(&mut self) -> Result<Stml, ()> {
        if self.check_consume(&[TokenType::Import])? {
            self.import_stml()
        } else if self.check_consume(&[TokenType::Function])? {
            todo!()
        } else if self.check_consume(&[TokenType::Var])? {
            todo!()
        } else if self.check_consume(&[TokenType::While])? {
            todo!()
        } else if self.check_consume(&[TokenType::Loop])? {
            todo!()
        } else if self.check_consume(&[TokenType::If])? {
            todo!()
        } else if self.check_consume(&[TokenType::Try])? {
            todo!()
        } else if self.check_consume(&[TokenType::OBrace])? {
            todo!()
        } else if self.check_consume(&[TokenType::Break])? {
            todo!()
        } else if self.check_consume(&[TokenType::Continue])? {
            todo!()
        } else if self.check_consume(&[TokenType::Return])? {
            todo!()
        } else if self.check_consume(&[TokenType::Throw])? {
            todo!()
        } else if self.check_consume(&[TokenType::Export])? {
            todo!()
        } else if self.check_consume(&[TokenType::For])? {
            todo!()
        } else {
            self.expr_stml()
        }
    }

    #[allow(unused_must_use)]
    fn sync(&mut self) {
        while !self.at_end().unwrap_or(false)
            && !self
                .check(&[
                    TokenType::Function,
                    TokenType::Var,
                    TokenType::While,
                    TokenType::Loop,
                    TokenType::If,
                    TokenType::Try,
                    TokenType::OBrace,
                    TokenType::Break,
                    TokenType::Continue,
                    TokenType::Return,
                    TokenType::Throw,
                    TokenType::Import,
                    TokenType::Export,
                    TokenType::For,
                ])
                .unwrap_or(false)
        {
            self.next();
        }
    }

    #[allow(unused_must_use)]
    pub fn parse(mut self) -> Result<Vec<Stml>, Vec<Error>> {
        println!("[PARSER] started");
        self.validate_current();
        let mut ast = vec![];
        while !self.at_end().unwrap_or(false) {
            match self.stml() {
                Ok(stml) => ast.push(stml),
                Err(_) => self.sync(),
            }
        }
        if self.errors.is_empty() {
            if cfg!(feature = "verbose") {
                println!("[PARSER] succeeded");
                println!("{ast:#?}")
            }
            Ok(ast)
        } else {
            if cfg!(feature = "verbose") {
                println!("[PARSER] failed")
            }
            Err(self.errors)
        }
    }
}
