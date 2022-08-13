use super::token::{Token, TokenInside};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Literal {
    /// token
    Number(Rc<Token>),
    /// token
    String(Rc<Token>),
    /// token
    Bool(Rc<Token>),
    /// token
    Nil(Rc<Token>),
    /// token, exprs
    List(Rc<Token>, Vec<Expr>),
    /// token, props: \[(key, value, default)\]
    Object(
        Rc<Token>,
        Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>,
    ),
    /// token, required, optional, variadic: (token, name), body
    Lambda(
        Rc<Token>,
        Vec<Expr>,
        Vec<(Expr, Expr)>,
        Option<(Rc<Token>, Box<Expr>)>,
        Box<Stml>,
    ),
}

impl TokenInside for Literal {
    fn token(&self) -> Rc<Token> {
        match self {
            Self::Number(token)
            | Self::String(token)
            | Self::Bool(token)
            | Self::Nil(token)
            | Self::List(token, ..)
            | Self::Object(token, ..)
            | Self::Lambda(token, ..) => Rc::clone(token),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    /// token
    Variable(Rc<Token>),
    /// literal
    Literal(Literal),
    /// op, expr
    Unary(Rc<Token>, Box<Expr>),
    /// lhs, op, rhs
    Binary(Box<Expr>, Rc<Token>, Box<Expr>),
    /// expr, op, exprs
    Call(Box<Expr>, Rc<Token>, Vec<Expr>),
    /// expr, op, key
    Member(Box<Expr>, Rc<Token>, Box<Expr>),
}

impl TokenInside for Expr {
    fn token(&self) -> Rc<Token> {
        match self {
            Self::Variable(token) => Rc::clone(token),
            Self::Unary(op, ..)
            | Self::Binary(_, op, ..)
            | Self::Call(_, op, ..)
            | Self::Member(_, op, ..) => Rc::clone(op),
            Self::Literal(literal) => literal.token(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stml {
    /// token, stmls
    Block(Rc<Token>, Vec<Stml>),
    /// export_token, token, name, required: \[definable\], optional: \[(definable, default)\], : (token, name): (token, name), body
    FunctionDecl(
        Option<Rc<Token>>,
        Rc<Token>,
        Rc<Token>,
        Vec<Expr>,
        Vec<(Expr, Expr)>,
        Option<(Rc<Token>, Box<Expr>)>,
        Box<Stml>,
    ),
    /// export_token, token, decls: \[(definable, init)\]
    VarDecl(Option<Rc<Token>>, Rc<Token>, Vec<(Expr, Option<Expr>)>),
    /// token, expr
    Return(Rc<Token>, Option<Expr>),
    /// token, expr
    Throw(Rc<Token>, Option<Expr>),
    /// token, body, catch_token, err, catch_body
    TryCatch(Rc<Token>, Box<Stml>, Rc<Token>, Rc<Token>, Box<Stml>),
    /// token, condition, body, elseifs: \[(token, condition, body)\], else_: (token, body)
    If(
        Rc<Token>,
        Expr,
        Box<Stml>,
        Vec<(Rc<Token>, Expr, Stml)>,
        Option<(Rc<Token>, Box<Stml>)>,
    ),
    /// token, condition, body
    While(Rc<Token>, Expr, Box<Stml>),
    /// token, body
    Loop(Rc<Token>, Box<Stml>),
    /// token
    Break(Rc<Token>),
    /// token
    Continue(Rc<Token>),
    /// token, definable, from_token, path
    Import(Rc<Token>, Expr, Rc<Token>, Rc<Token>),
    /// token, definable, in_token, iterable, body
    ForIn(Rc<Token>, Expr, Rc<Token>, Expr, Box<Stml>),
    /// expr
    Expr(Expr),
}

impl TokenInside for Stml {
    fn token(&self) -> Rc<Token> {
        match self {
            Self::Block(token, ..)
            | Self::FunctionDecl(_, token, ..)
            | Self::VarDecl(_, token, ..)
            | Self::Return(token, ..)
            | Self::Throw(token, ..)
            | Self::TryCatch(token, ..)
            | Self::If(token, ..)
            | Self::While(token, ..)
            | Self::Loop(token, ..)
            | Self::Break(token)
            | Self::Continue(token)
            | Self::Import(token, ..)
            | Self::ForIn(token, ..) => Rc::clone(token),
            Self::Expr(expr) => expr.token(),
        }
    }
}
