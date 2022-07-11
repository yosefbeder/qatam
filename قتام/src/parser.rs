use super::lexer::LexicalError;
use super::{
    ast::{Expr, Literal, Stml},
    lexer::{self, Lexer},
    operators::{Associativity, OPERATORS},
    token::{Token, TokenType, BINARY_SET, BOUNDARIES},
};
use std::vec;
use std::{fmt, path::PathBuf, rc::Rc, result};

type Result<T> = result::Result<T, ()>;

#[derive(Debug, Clone)]
enum ParseError {
    ExpectedInstead(Vec<TokenType>, Token),
    ExpectedExpr(Token),
    InvalidRhs(Token),
}

#[derive(PartialEq, Clone, Copy)]
enum AssignAbility {
    AnyOp,
    OnlyEqual,
    None,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpectedInstead(expected, token) => {
                let got: &str = token.typ.to_owned().into();
                write!(
                    f,
                    "توقعت {} ولكن حصلت على '{got} ' {}",
                    expected
                        .iter()
                        .map(|typ| {
                            let as_str: &str = typ.to_owned().into();
                            format!("'{as_str}'")
                        })
                        .collect::<Vec<_>>()
                        .join(" أو "),
                    token.get_pos(),
                )
            }
            Self::ExpectedExpr(token) => {
                let got: &str = token.typ.to_owned().into();
                write!(f, "توقعت عبارة ولكن حصلت على '{got}' {}", token.get_pos())
            }
            Self::InvalidRhs(token) => {
                write!(
                    f,
                    "الجانب الأيمن لعلامة التساوي غير صحيح {}",
                    token.get_pos(),
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    Lexical(LexicalError),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lexical(err) => write!(f, "{err}"),
            Self::Parse(err) => write!(f, "{err}"),
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    current: lexer::Result,
    previous: Option<Token>,
    had_err: bool,
}

impl Parser {
    pub fn new(source: String, path: Option<PathBuf>) -> Self {
        let mut lexer = Lexer::new(source, path);
        let current = lexer.next_token();

        Self {
            lexer,
            current,
            previous: None,
            had_err: false,
        }
    }

    fn err(&mut self, err: Error) {
        self.had_err = true;
        eprintln!("{err}");
    }

    fn lexical_err(&mut self, err: LexicalError) {
        self.err(Error::Lexical(err))
    }

    fn parse_err(&mut self, err: ParseError) {
        self.err(Error::Parse(err))
    }

    fn current_token(&self) -> &Token {
        match &self.current {
            Ok(token) => token,
            Err(err) => err.get_token(),
        }
    }

    /// makes `self.previous` contain a valid token
    fn advance(&mut self) -> Result<()> {
        loop {
            if let Err(err) = self.current.clone() {
                self.lexical_err(err);
                self.current = self.lexer.next_token();
                return Err(());
            }
            if [TokenType::NewLine, TokenType::Comment].contains(&self.current_token().typ) {
                self.current = self.lexer.next_token();
                continue;
            }
            break;
        }
        self.previous = Some(self.current_token().clone());
        self.current = self.lexer.next_token();
        Ok(())
    }

    /// may return an invalid token
    fn peek(&mut self, ignore_newlines: bool) -> Token {
        loop {
            if self.current_token().typ == TokenType::Comment
                || ignore_newlines && self.current_token().typ == TokenType::NewLine
            {
                self.current = self.lexer.next_token();
                continue;
            }
            break;
        }

        self.current_token().clone()
    }

    fn check(&mut self, typ: TokenType) -> bool {
        let ignore_newlines = typ != TokenType::NewLine;

        if self.peek(ignore_newlines).typ == typ {
            return true;
        }

        false
    }

    fn check_consume(&mut self, typ: TokenType) -> bool {
        if self.check(typ) {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Result<Token> {
        self.advance()?;
        Ok(self.clone_previous())
    }

    fn consume(&mut self, typ: TokenType) -> Result<()> {
        if self.check_consume(typ) {
            Ok(())
        } else {
            let token = self.current_token().clone();
            self.parse_err(ParseError::ExpectedInstead(vec![typ], token));
            Err(())
        }
    }

    fn at_end(&mut self) -> bool {
        self.check(TokenType::EOF)
    }

    fn clone_previous(&self) -> Token {
        self.previous.as_ref().unwrap().clone()
    }

    fn exprs(&mut self, closing_token: TokenType) -> Result<Vec<Expr>> {
        if self.check_consume(closing_token) {
            return Ok(vec![]);
        }
        let mut items = vec![self.parse_expr()?];
        while self.check_consume(TokenType::Comma) {
            if self.check_consume(closing_token) {
                return Ok(items);
            }
            items.push(self.parse_expr()?);
        }
        self.consume(closing_token)?;
        Ok(items)
    }

    fn list(&mut self) -> Result<Expr> {
        Ok(Expr::Literal(Literal::List(
            self.exprs(TokenType::CBracket)?,
        )))
    }

    fn prop(&mut self) -> Result<(Rc<Token>, Option<Expr>)> {
        self.consume(TokenType::Identifier)?;
        let key = self.clone_previous();
        let value = if self.check_consume(TokenType::Colon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok((Rc::new(key), value))
    }

    fn props(&mut self) -> Result<Vec<(Rc<Token>, Option<Expr>)>> {
        if self.check_consume(TokenType::CBrace) {
            return Ok(vec![]);
        }
        let mut items = vec![self.prop()?];
        while self.check_consume(TokenType::Comma) {
            if self.check_consume(TokenType::CBrace) {
                return Ok(items);
            }
            items.push(self.prop()?);
        }
        self.consume(TokenType::CBrace)?;
        Ok(items)
    }

    fn object(&mut self) -> Result<Expr> {
        Ok(Expr::Literal(Literal::Object(self.props()?)))
    }

    fn lambda(&mut self) -> Result<Expr> {
        let token = self.clone_previous();
        if token.typ == TokenType::Or {
            self.consume(TokenType::OBrace)?;
            let body = self.block()?;
            Ok(Expr::Lambda(Rc::new(token), vec![], Box::new(body)))
        } else {
            let params = self.params(TokenType::Pipe)?;
            self.consume(TokenType::OBrace)?;
            let body = self.block()?;
            Ok(Expr::Lambda(Rc::new(token), params, Box::new(body)))
        }
    }

    fn literal(&mut self) -> Result<Expr> {
        let token = self.clone_previous();
        match token.typ {
            TokenType::Identifier => Ok(Expr::Variable(Rc::new(token))),
            TokenType::Number => Ok(Expr::Literal(Literal::Number(Rc::new(token)))),
            TokenType::String => Ok(Expr::Literal(Literal::String(Rc::new(token)))),
            TokenType::True | TokenType::False => Ok(Expr::Literal(Literal::Bool(Rc::new(token)))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil(Rc::new(token)))),
            TokenType::OBracket => self.list(),
            TokenType::OBrace => self.object(),
            TokenType::Pipe | TokenType::Or => self.lambda(),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        let token = self.clone_previous();
        let row: usize = token.typ.into();
        let prefix_precedence = OPERATORS[row].0.unwrap();
        let rhs = self.expr(prefix_precedence, AssignAbility::None)?;
        Ok(Expr::Unary(Rc::new(token), Box::new(rhs)))
    }

    fn group(&mut self) -> Result<Expr> {
        let expr = self.parse_expr()?;
        self.consume(TokenType::CParen)?;
        return Ok(expr);
    }

    fn can_assign(typ: TokenType, assign_ops: AssignAbility) -> bool {
        match typ {
            TokenType::Equal => assign_ops != AssignAbility::None,
            x if BINARY_SET.contains(&x) => assign_ops == AssignAbility::AnyOp,
            _ => false,
        }
    }

    /// Parses any expression with a binding power more than or equal to `min_bp`.
    fn expr(&mut self, min_precedence: u8, mut assign_abililty: AssignAbility) -> Result<Expr> {
        let mut token = self.next()?;
        let mut expr;

        expr = match token.typ {
            TokenType::Identifier => self.literal()?,
            TokenType::OBracket | TokenType::OBrace => {
                assign_abililty = AssignAbility::OnlyEqual;
                self.literal()?
            }
            TokenType::Number
            | TokenType::String
            | TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::Pipe
            | TokenType::Or => {
                assign_abililty = AssignAbility::None;
                self.literal()?
            }
            TokenType::Minus | TokenType::Bang => {
                assign_abililty = AssignAbility::None;
                self.unary()?
            }
            TokenType::OParen => {
                assign_abililty = AssignAbility::None;
                self.group()?
            }
            _ => {
                self.parse_err(ParseError::ExpectedExpr(token));
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

                self.advance()?;

                if !BINARY_SET.contains(&token.typ) {
                    assign_abililty = AssignAbility::None;
                }

                let can_assign = Self::can_assign(token.typ, assign_abililty);

                if BINARY_SET.contains(&token.typ) && !can_assign {
                    self.parse_err(ParseError::InvalidRhs(token.clone()));
                }

                expr = Expr::Binary(
                    Rc::new(token),
                    Box::new(expr),
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

                match token.typ {
                    TokenType::OParen => {
                        assign_abililty = AssignAbility::None;
                        expr = Expr::Call(
                            Rc::new(token),
                            Box::new(expr),
                            self.exprs(TokenType::CParen)?,
                        );
                    }
                    TokenType::Period | TokenType::OBracket => {
                        match expr {
                            Expr::Call(_, _, _) => {
                                assign_abililty = AssignAbility::AnyOp;
                            }
                            _ => {}
                        }

                        let key = match token.typ {
                            TokenType::Period => {
                                self.consume(TokenType::Identifier)?;
                                Expr::Literal(Literal::String(Rc::new(self.clone_previous())))
                            }
                            TokenType::OBracket => {
                                let tmp = self.parse_expr()?;
                                self.consume(TokenType::CBracket)?;
                                tmp
                            }
                            _ => unreachable!(),
                        };

                        expr = Expr::Member(Rc::new(token), Box::new(expr), Box::new(key));
                    }
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn block(&mut self) -> Result<Stml> {
        let mut stmls = vec![];
        if !self.check(TokenType::CBrace) {
            while !self.at_end() && !self.check(TokenType::CBrace) {
                stmls.push(self.stml()?);
            }
        };
        self.consume(TokenType::CBrace)?;
        Ok(Stml::Block(stmls))
    }

    fn return_stml(&mut self) -> Result<Stml> {
        let token = self.clone_previous();
        if self.check(TokenType::NewLine) {
            return Ok(Stml::Return(Rc::new(token), None));
        }
        Ok(Stml::Return(Rc::new(token), Some(self.parse_expr()?)))
    }

    fn throw_stml(&mut self) -> Result<Stml> {
        let token = self.clone_previous();
        if self.check(TokenType::NewLine) {
            return Ok(Stml::Throw(Rc::new(token), None));
        }
        Ok(Stml::Throw(Rc::new(token), Some(self.parse_expr()?)))
    }

    fn params(&mut self, closing_token: TokenType) -> Result<Vec<Expr>> {
        if self.check_consume(closing_token) {
            return Ok(vec![]);
        }
        let mut items = vec![self.definable()?];
        while self.check_consume(TokenType::Comma) {
            if self.check_consume(closing_token) {
                return Ok(items);
            }
            items.push(self.definable()?);
        }
        self.consume(closing_token)?;
        Ok(items)
    }

    fn function_decl(&mut self) -> Result<Stml> {
        self.consume(TokenType::Identifier)?;
        let name = self.clone_previous();
        self.consume(TokenType::OParen)?;
        let params = self.params(TokenType::CParen)?;
        self.consume(TokenType::OBrace)?;
        let body = self.block()?;
        Ok(Stml::FunctionDecl(Rc::new(name), params, Box::new(body)))
    }

    fn expr_stml(&mut self) -> Result<Stml> {
        Ok(Stml::Expr(self.parse_expr()?))
    }

    fn while_stml(&mut self) -> Result<Stml> {
        self.consume(TokenType::OParen)?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::CParen)?;
        self.consume(TokenType::OBrace)?;
        let body = self.block()?;
        Ok(Stml::While(condition, Box::new(body)))
    }

    fn loop_stml(&mut self) -> Result<Stml> {
        self.consume(TokenType::OBrace)?;
        let body = self.block()?;
        Ok(Stml::Loop(Box::new(body)))
    }

    fn break_stml(&mut self) -> Result<Stml> {
        Ok(Stml::Break(Rc::new(self.clone_previous())))
    }

    fn continue_stml(&mut self) -> Result<Stml> {
        Ok(Stml::Continue(Rc::new(self.clone_previous())))
    }

    fn try_catch(&mut self) -> Result<Stml> {
        self.consume(TokenType::OBrace)?;
        let body = self.block()?;
        self.consume(TokenType::Catch)?;
        self.consume(TokenType::OParen)?;
        self.consume(TokenType::Identifier)?;
        let name = self.clone_previous();
        self.consume(TokenType::CParen)?;
        self.consume(TokenType::OBrace)?;
        let catch_body = self.block()?;
        Ok(Stml::TryCatch(
            Box::new(body),
            Rc::new(name),
            Box::new(catch_body),
        ))
    }

    fn if_else_stml(&mut self) -> Result<Stml> {
        self.consume(TokenType::OParen)?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::CParen)?;
        self.consume(TokenType::OBrace)?;
        let if_body = self.block()?;

        let mut elseifs = vec![];
        while self.check_consume(TokenType::ElseIf) {
            self.consume(TokenType::OParen)?;
            let condition = self.parse_expr()?;
            self.consume(TokenType::CParen)?;

            self.consume(TokenType::OBrace)?;
            let body = self.block()?;

            elseifs.push((condition, body));
        }

        let else_body = if self.check_consume(TokenType::Else) {
            self.consume(TokenType::OBrace)?;
            Some(Box::new(self.block()?))
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

    fn definable(&mut self) -> Result<Expr> {
        Ok(if self.check_consume(TokenType::Identifier) {
            Expr::Variable(Rc::new(self.clone_previous()))
        } else if self.check_consume(TokenType::OBracket) {
            self.list()?
        } else if self.check_consume(TokenType::OBrace) {
            self.object()?
        } else {
            self.err(Error::Parse(ParseError::ExpectedInstead(
                vec![
                    TokenType::Identifier,
                    TokenType::OBracket,
                    TokenType::OBrace,
                ],
                self.current_token().clone(),
            )));
            return Err(());
        })
    }

    fn var_decl(&mut self) -> Result<Stml> {
        let token = self.clone_previous();
        let definable = self.definable()?;
        let initializer = if !definable.is_variable() {
            self.consume(TokenType::Equal)?;
            Some(self.parse_expr()?)
        } else {
            if self.check_consume(TokenType::Equal) {
                Some(self.parse_expr()?)
            } else {
                None
            }
        };
        Ok(Stml::VarDecl(Rc::new(token), definable, initializer))
    }

    fn for_in(&mut self) -> Result<Stml> {
        let token = self.clone_previous();
        self.consume(TokenType::OParen)?;
        let definable = self.definable()?;
        self.consume(TokenType::In)?;
        let iterable = self.parse_expr()?;
        self.consume(TokenType::CParen)?;
        self.consume(TokenType::OBrace)?;
        let body = self.block()?;
        Ok(Stml::ForIn(
            Rc::new(token),
            definable,
            iterable,
            Box::new(body),
        ))
    }

    fn import_stml(&mut self) -> Result<Stml> {
        let token = self.clone_previous();
        let definable = self.definable()?;
        self.consume(TokenType::From)?;
        self.consume(TokenType::String)?;
        Ok(Stml::Import(
            Rc::new(token),
            definable,
            Rc::new(self.clone_previous()),
        ))
    }

    fn export_stml(&mut self) -> Result<Stml> {
        let token = self.clone_previous();

        if self.check_consume(TokenType::Function) {
            Ok(Stml::Export(
                Rc::new(token),
                Box::new(self.function_decl()?),
            ))
        } else if self.check_consume(TokenType::Var) {
            Ok(Stml::Export(Rc::new(token), Box::new(self.var_decl()?)))
        } else {
            let token = self.current_token().clone();
            self.parse_err(ParseError::ExpectedInstead(
                vec![TokenType::Function, TokenType::Var],
                token,
            ));
            Err(())
        }
    }

    fn stml(&mut self) -> Result<Stml> {
        if self.check_consume(TokenType::Function) {
            self.function_decl()
        } else if self.check_consume(TokenType::Var) {
            self.var_decl()
        } else if self.check_consume(TokenType::Export) {
            self.export_stml()
        } else if self.check_consume(TokenType::Import) {
            self.import_stml()
        } else if self.check_consume(TokenType::While) {
            self.while_stml()
        } else if self.check_consume(TokenType::Loop) {
            self.loop_stml()
        } else if self.check_consume(TokenType::If) {
            self.if_else_stml()
        } else if self.check_consume(TokenType::Try) {
            self.try_catch()
        } else if self.check_consume(TokenType::OBrace) {
            self.block()
        } else if self.check_consume(TokenType::Break) {
            self.break_stml()
        } else if self.check_consume(TokenType::Continue) {
            self.continue_stml()
        } else if self.check_consume(TokenType::Return) {
            self.return_stml()
        } else if self.check_consume(TokenType::Throw) {
            self.throw_stml()
        } else if self.check_consume(TokenType::For) {
            self.for_in()
        } else {
            self.expr_stml()
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

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.expr(9, AssignAbility::AnyOp)
    }

    pub fn parse(&mut self) -> Result<Vec<Stml>> {
        if cfg!(feature = "debug-ast") {
            println!("---");
            println!("[DEBUG] started parsing");
            println!("---");
        }

        let mut stmls = vec![];
        while !self.at_end() {
            match self.stml() {
                Ok(stml) => stmls.push(stml),
                Err(_) => {
                    self.sync();
                }
            }
        }
        if self.had_err {
            Err(())
        } else {
            Ok(stmls)
        }
    }
}
