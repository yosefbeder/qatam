pub mod ast;
mod lexer;
mod operators;
pub mod token;

use ast::*;
use colored::Colorize;
use lexer::{Lexer, LexicalError};
use operators::*;
use std::{fmt, path::PathBuf, rc::Rc, string};
use token::*;
use TokenType::*;

const BINARY_SET: [TokenType; 6] = [
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
];

#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectedInstead(Vec<TokenType>, Rc<Token>),
    ExpectedExpr(Rc<Token>),
    InvalidRhs(Rc<Token>),
    ExpectedOptional(Rc<Token>),
}

use ParseError::*;

#[derive(PartialEq, Clone, Copy)]
enum AssignAbility {
    AnyOp,
    OnlyEqual,
    None,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "خطأ تحليلي: ".bright_red())?;
        match self {
            ExpectedInstead(expected, token) => {
                let got: &str = token.typ().to_owned().into();
                write!(
                    f,
                    "توقعت {} ولكن حصلت على \"{got}\"\n{token}",
                    expected
                        .iter()
                        .map(|typ| {
                            let as_str: &str = typ.to_owned().into();
                            format!("\"{as_str}\"")
                        })
                        .collect::<Vec<_>>()
                        .join(" أو "),
                )
            }
            ExpectedExpr(token) => {
                let got: &str = token.typ().to_owned().into();
                write!(f, "توقعت عبارة ولكن حصلت على \"{got}\"\n{token}")
            }
            InvalidRhs(token) => {
                write!(f, "الجانب الأيمن لعلامة التساوي غير صحيح\n{token}")
            }
            ExpectedOptional(token) => {
                write!(f, "لا يمكن وضع مدخل إجباري بعد مدخل إختياري\n{token}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
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
    previous: Option<Rc<Token>>,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(source: string::String, path: Option<PathBuf>) -> Self {
        let mut lexer = Lexer::new(source, path);
        let current = lexer.next_token();

        Self {
            lexer,
            current,
            previous: None,
            errors: vec![],
        }
    }

    fn err(&mut self, err: Error) {
        self.errors.push(err);
    }

    fn lexical_err(&mut self, err: LexicalError) {
        self.err(Error::Lexical(err))
    }

    fn parse_err(&mut self, err: ParseError) {
        self.err(Error::Parse(err))
    }

    fn current_token(&self) -> Rc<Token> {
        match &self.current {
            Ok(token) => Rc::clone(token),
            Err(err) => err.token(),
        }
    }

    /// Makes `self.previous` contain a valid token.
    fn advance(&mut self) -> Result<(), ()> {
        loop {
            if let Err(err) = self.current.clone() {
                self.lexical_err(err);
                self.current = self.lexer.next_token();
                return Err(());
            }
            if [NewLine, Comment].contains(&self.current_token().typ()) {
                self.current = self.lexer.next_token();
                continue;
            }
            break;
        }
        self.previous = Some(self.current_token());
        self.current = self.lexer.next_token();
        Ok(())
    }

    /// May return an invalid token.
    fn peek(&mut self, ignore_newlines: bool) -> Rc<Token> {
        loop {
            if self.current_token().typ() == Comment
                || ignore_newlines && self.current_token().typ() == NewLine
            {
                self.current = self.lexer.next_token();
                continue;
            }
            break;
        }

        Rc::clone(&self.current_token())
    }

    fn check(&mut self, typ: TokenType) -> bool {
        let ignore_newlines = typ != NewLine;

        if self.peek(ignore_newlines).typ() == typ {
            return true;
        }

        if !ignore_newlines && self.at_end() {
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

    fn next(&mut self) -> Result<Rc<Token>, ()> {
        self.advance()?;
        Ok(self.clone_previous())
    }

    fn consume(&mut self, typ: TokenType) -> Result<(), ()> {
        if self.check_consume(typ) {
            Ok(())
        } else {
            let token = self.current_token();
            self.parse_err(ExpectedInstead(vec![typ], token));
            Err(())
        }
    }

    fn at_end(&mut self) -> bool {
        self.check(EOF)
    }

    fn clone_previous(&self) -> Rc<Token> {
        Rc::clone(self.previous.as_ref().unwrap())
    }

    fn can_assign(typ: TokenType, assign_ops: AssignAbility) -> bool {
        match typ {
            Equal => assign_ops != AssignAbility::None,
            x if BINARY_SET.contains(&x) => assign_ops == AssignAbility::AnyOp,
            _ => false,
        }
    }

    fn exprs(&mut self, closing_token: TokenType) -> Result<Vec<Expr>, ()> {
        if self.check_consume(closing_token) {
            return Ok(vec![]);
        }
        let mut items = vec![self.parse_expr()?];
        while self.check_consume(Comma) {
            if self.check_consume(closing_token) {
                return Ok(items);
            }
            items.push(self.parse_expr()?);
        }
        self.consume(closing_token)?;
        Ok(items)
    }

    fn list(&mut self) -> Result<Expr, ()> {
        Ok(Expr::Literal(Literal::List(
            self.clone_previous(),
            self.exprs(CBracket)?,
        )))
    }

    fn prop(&mut self) -> Result<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>), ()> {
        self.consume(Identifier)?;
        let key = self.clone_previous();
        let mut value = if self.check_consume(Colon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let default = match value.clone() {
            Some(Expr::Binary(lhs, op, rhs)) if op.typ() == Equal => {
                value = Some(*lhs);
                Some((op, *rhs))
            }
            None => {
                if self.check_consume(Equal) {
                    Some((self.clone_previous(), self.parse_expr()?))
                } else {
                    None
                }
            }
            _ => None,
        };
        Ok((key, value, default))
    }

    fn props(&mut self) -> Result<Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>, ()> {
        if self.check_consume(CBrace) {
            return Ok(vec![]);
        }
        let mut tmp = vec![self.prop()?];
        while self.check_consume(Comma) {
            if self.check_consume(CBrace) {
                return Ok(tmp);
            }
            tmp.push(self.prop()?);
        }
        self.consume(CBrace)?;
        Ok(tmp)
    }

    fn object(&mut self) -> Result<Expr, ()> {
        Ok(Expr::Literal(Literal::Object(
            self.clone_previous(),
            self.props()?,
        )))
    }

    fn lambda(&mut self) -> Result<Expr, ()> {
        let token = self.clone_previous();
        let (required, optional, variadic) = self.params(Pipe)?;
        self.consume(OBrace)?;
        let body = self.block()?;
        Ok(Expr::Literal(Literal::Lambda(
            token,
            required,
            optional,
            variadic,
            Box::new(body),
        )))
    }

    fn literal(&mut self) -> Result<Expr, ()> {
        let token = self.clone_previous();
        match token.typ() {
            Identifier => Ok(Expr::Variable(token)),
            Number => Ok(Expr::Literal(Literal::Number(token))),
            String => Ok(Expr::Literal(Literal::String(token))),
            True | False => Ok(Expr::Literal(Literal::Bool(token))),
            Nil => Ok(Expr::Literal(Literal::Nil(token))),
            OBracket => self.list(),
            OBrace => self.object(),
            Pipe => self.lambda(),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        let op = self.clone_previous();
        let row: usize = op.typ() as usize;
        let prefix_precedence = OPERATORS[row].0.unwrap();
        let expr = self.expr(prefix_precedence, AssignAbility::None)?;
        Ok(Expr::Unary(op, Box::new(expr)))
    }

    fn group(&mut self) -> Result<Expr, ()> {
        let expr = self.parse_expr()?;
        self.consume(CParen)?;
        return Ok(expr);
    }

    /// Parses any expression with a binding power more than or equal to `min_bp`.
    fn expr(&mut self, min_precedence: u8, mut assign_abililty: AssignAbility) -> Result<Expr, ()> {
        let token = self.next()?;
        let mut expr;

        expr = match token.typ() {
            Identifier => self.literal()?,
            OBracket | OBrace => {
                assign_abililty = AssignAbility::OnlyEqual;
                self.literal()?
            }
            Number | String | True | False | Nil | Pipe | Or => {
                assign_abililty = AssignAbility::None;
                self.literal()?
            }
            Minus | Bang => {
                assign_abililty = AssignAbility::None;
                self.unary()?
            }
            OParen => {
                assign_abililty = AssignAbility::None;
                self.group()?
            }
            _ => {
                self.parse_err(ExpectedExpr(token));
                return Err(());
            }
        };

        while !self.check(NewLine) && !self.at_end() {
            let op = self.peek(true);
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
                    self.parse_err(InvalidRhs(Rc::clone(&op)));
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
                    OParen => {
                        assign_abililty = AssignAbility::None;
                        expr = Expr::Call(Box::new(expr), op, self.exprs(CParen)?);
                    }
                    Period | OBracket => {
                        match expr {
                            Expr::Call(_, _, _) => {
                                assign_abililty = AssignAbility::AnyOp;
                            }
                            _ => {}
                        }
                        let key = match op.typ() {
                            Period => {
                                self.consume(Identifier)?;
                                Expr::Literal(Literal::String(self.clone_previous()))
                            }
                            OBracket => {
                                let tmp = self.parse_expr()?;
                                self.consume(CBracket)?;
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

    fn block(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        let mut stmls = vec![];
        if !self.check(CBrace) {
            while !self.at_end() && !self.check(CBrace) {
                stmls.push(self.stml()?);
            }
        };
        self.consume(CBrace)?;
        Ok(Stml::Block(token, stmls))
    }

    fn return_stml(&mut self) -> Result<Stml, ()> {
        Ok(Stml::Return(
            self.clone_previous(),
            if self.check(NewLine) {
                None
            } else {
                Some(self.parse_expr()?)
            },
        ))
    }

    fn throw_stml(&mut self) -> Result<Stml, ()> {
        Ok(Stml::Throw(
            self.clone_previous(),
            if self.check(NewLine) {
                None
            } else {
                Some(self.parse_expr()?)
            },
        ))
    }

    /// Parses a required or optional param.
    ///
    /// If nothing is returned, this implies that it either found "..." or `closing_token`.
    fn param(&mut self, closing_token: TokenType) -> Result<Option<(Expr, Option<Expr>)>, ()> {
        if self.check(TPeriod) || self.check(closing_token) {
            return Ok(None);
        }
        let definable = self.definable()?;
        if self.check_consume(Equal) {
            Ok(Some((definable, Some(self.parse_expr()?))))
        } else {
            Ok(Some((definable, None)))
        }
    }

    fn variadic_param(&mut self) -> Result<(Rc<Token>, Box<Expr>), ()> {
        let token = self.clone_previous();
        self.consume(Identifier)?;
        Ok((token, Box::new(Expr::Variable(self.clone_previous()))))
    }

    fn params(
        &mut self,
        closing_token: TokenType,
    ) -> Result<(Vec<Expr>, Vec<(Expr, Expr)>, Option<(Rc<Token>, Box<Expr>)>), ()> {
        let mut required = vec![];
        let mut optional = vec![];

        macro_rules! optional {
            ($definable:ident, $default:ident) => {{
                optional.push(($definable, $default));
                while self.check_consume(Comma) {
                    match self.param(closing_token)? {
                        Some((definable, None)) => {
                            self.parse_err(ExpectedOptional(definable.token()))
                        }
                        Some((definable, Some(default))) => {
                            optional.push((definable, default));
                        }
                        None => {
                            break;
                        }
                    }
                }
            }};
        }

        match self.param(closing_token)? {
            Some((definable, None)) => {
                required.push(definable);
                while self.check_consume(Comma) {
                    match self.param(closing_token)? {
                        Some((definable, None)) => {
                            required.push(definable);
                        }
                        Some((definable, Some(default))) => {
                            optional!(definable, default);
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
            Some((definable, Some(default))) => {
                optional!(definable, default)
            }
            None => {}
        }

        let variadic = if self.check_consume(TPeriod) {
            let tmp = self.variadic_param()?;
            self.check_consume(Comma);
            Some(tmp)
        } else {
            None
        };
        self.consume(closing_token)?;
        Ok((required, optional, variadic))
    }

    fn function_decl(&mut self, export_token: Option<Rc<Token>>) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(Identifier)?;
        let name = self.clone_previous();
        self.consume(OParen)?;
        let (required, optional, variadic) = self.params(CParen)?;
        self.consume(OBrace)?;
        let body = self.block()?;
        Ok(Stml::FunctionDecl(
            export_token,
            token,
            name,
            required,
            optional,
            variadic,
            Box::new(body),
        ))
    }

    fn while_stml(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(OParen)?;
        let condition = self.parse_expr()?;
        self.consume(CParen)?;
        self.consume(OBrace)?;
        let body = self.block()?;
        Ok(Stml::While(token, condition, Box::new(body)))
    }

    fn loop_stml(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(OBrace)?;
        let body = self.block()?;
        Ok(Stml::Loop(token, Box::new(body)))
    }

    fn try_catch(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(OBrace)?;
        let body = self.block()?;
        self.consume(Catch)?;
        let catch_token = self.clone_previous();
        self.consume(OParen)?;
        self.consume(Identifier)?;
        let err = self.clone_previous();
        self.consume(CParen)?;
        self.consume(OBrace)?;
        let catch_body = self.block()?;
        Ok(Stml::TryCatch(
            token,
            Box::new(body),
            catch_token,
            err,
            Box::new(catch_body),
        ))
    }

    fn if_else_stml(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(OParen)?;
        let condition = self.parse_expr()?;
        self.consume(CParen)?;
        self.consume(OBrace)?;
        let if_body = self.block()?;
        let mut elseifs = vec![];
        while self.check_consume(ElseIf) {
            let token = self.clone_previous();
            self.consume(OParen)?;
            let condition = self.parse_expr()?;
            self.consume(CParen)?;
            self.consume(OBrace)?;
            let body = self.block()?;
            elseifs.push((token, condition, body));
        }
        let else_body = if self.check_consume(Else) {
            let token = self.clone_previous();
            self.consume(OBrace)?;
            Some((token, Box::new(self.block()?)))
        } else {
            None
        };
        Ok(Stml::If(
            token,
            condition,
            Box::new(if_body),
            elseifs,
            else_body,
        ))
    }

    fn definable(&mut self) -> Result<Expr, ()> {
        Ok(if self.check_consume(Identifier) {
            Expr::Variable(self.clone_previous())
        } else if self.check_consume(OBracket) {
            self.list()?
        } else if self.check_consume(OBrace) {
            self.object()?
        } else {
            self.err(Error::Parse(ExpectedInstead(
                vec![Identifier, OBracket, OBrace],
                self.current_token(),
            )));
            return Err(());
        })
    }

    fn var_decl(&mut self) -> Result<(Expr, Option<Expr>), ()> {
        let definable = self.definable()?;
        let init;
        match definable {
            Expr::Variable(_) => {
                init = if self.check_consume(Equal) {
                    Some(self.parse_expr()?)
                } else {
                    None
                }
            }
            _ => {
                self.consume(Equal)?;
                init = Some(self.parse_expr()?)
            }
        }
        Ok((definable, init))
    }

    fn vars_decl(&mut self, export_token: Option<Rc<Token>>) -> Result<Stml, ()> {
        let token = self.clone_previous();
        let mut decls = vec![self.var_decl()?];

        while self.check_consume(Comma) {
            decls.push(self.var_decl()?);
        }

        Ok(Stml::VarDecl(export_token, token, decls))
    }

    fn for_in(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        self.consume(OParen)?;
        let definable = self.definable()?;
        self.consume(In)?;
        let in_token = self.clone_previous();
        let iterable = self.parse_expr()?;
        self.consume(CParen)?;
        self.consume(OBrace)?;
        let body = self.block()?;
        Ok(Stml::ForIn(
            token,
            definable,
            in_token,
            iterable,
            Box::new(body),
        ))
    }

    fn import_stml(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        let definable = self.definable()?;
        self.consume(From)?;
        let from_token = self.clone_previous();
        self.consume(String)?;
        Ok(Stml::Import(
            token,
            definable,
            from_token,
            self.clone_previous(),
        ))
    }

    fn export_stml(&mut self) -> Result<Stml, ()> {
        let token = self.clone_previous();
        if self.check_consume(Function) {
            Ok(self.function_decl(Some(token))?)
        } else if self.check_consume(Var) {
            Ok(self.vars_decl(Some(token))?)
        } else {
            let token = self.current_token().clone();
            self.parse_err(ExpectedInstead(vec![Function, Var], token));
            Err(())
        }
    }

    fn stml(&mut self) -> Result<Stml, ()> {
        if self.check_consume(Function) {
            self.function_decl(None)
        } else if self.check_consume(Var) {
            self.vars_decl(None)
        } else if self.check_consume(Export) {
            self.export_stml()
        } else if self.check_consume(Import) {
            self.import_stml()
        } else if self.check_consume(While) {
            self.while_stml()
        } else if self.check_consume(Loop) {
            self.loop_stml()
        } else if self.check_consume(If) {
            self.if_else_stml()
        } else if self.check_consume(Try) {
            self.try_catch()
        } else if self.check_consume(OBrace) {
            self.block()
        } else if self.check_consume(Break) {
            Ok(Stml::Break(self.clone_previous()))
        } else if self.check_consume(Continue) {
            Ok(Stml::Continue(self.clone_previous()))
        } else if self.check_consume(Return) {
            self.return_stml()
        } else if self.check_consume(Throw) {
            self.throw_stml()
        } else if self.check_consume(For) {
            self.for_in()
        } else {
            Ok(Stml::Expr(self.parse_expr()?))
        }
    }

    fn sync(&mut self) {
        while !self.check(EOF) {
            if [
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
            ]
            .contains(&self.peek(true).typ())
            {
                break;
            }
            self.advance().ok();
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ()> {
        self.expr(9, AssignAbility::AnyOp)
    }

    pub fn parse(&mut self) -> Result<(Vec<Stml>, Rc<Token>), Vec<Error>> {
        if cfg!(feature = "verbose") {
            println!("[PARSER] started")
        }

        let mut ast = vec![];
        while !self.at_end() {
            match self.stml() {
                Ok(stml) => ast.push(stml),
                Err(_) => {
                    self.sync();
                }
            }
        }
        if self.errors.len() > 0 {
            if cfg!(feature = "verbose") {
                println!("[PARSER] failed")
            }
            Err(self.errors.clone())
        } else {
            if cfg!(feature = "verbose") {
                println!("[PARSER] succeeded");
                println!("{ast:#?}")
            }
            Ok((ast, self.current_token()))
        }
    }
}
