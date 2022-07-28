use lexer::token::Token;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Rc<Token>),
    String(Rc<Token>),
    Bool(Rc<Token>),
    Nil(Rc<Token>),
    List(Rc<Token>, Vec<Expr>),
    Object(
        Rc<Token>,
        Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>,
    ),
    Lambda(
        Rc<Token>,
        Vec<Expr>,
        Vec<(Expr, Expr)>,
        Option<(Rc<Token>, Box<Expr>)>,
        Box<Stml>,
    ),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Variable(Rc<Token>),
    Literal(Literal),
    Unary(Rc<Token>, Box<Expr>),
    Binary(Rc<Token>, Box<Expr>, Box<Expr>),
    Call(Rc<Token>, Box<Expr>, Vec<Expr>),
    Member(Rc<Token>, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn token(&self) -> Rc<Token> {
        match self {
            Self::Variable(token)
            | Self::Literal(Literal::Number(token))
            | Self::Literal(Literal::String(token))
            | Self::Literal(Literal::Bool(token))
            | Self::Literal(Literal::Nil(token))
            | Self::Literal(Literal::List(token, ..))
            | Self::Literal(Literal::Object(token, ..))
            | Self::Literal(Literal::Lambda(token, ..)) => Rc::clone(token),
            Self::Unary(oper, ..)
            | Self::Binary(oper, ..)
            | Self::Call(oper, ..)
            | Self::Member(oper, ..) => Rc::clone(oper),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stml {
    Block(Rc<Token>, Vec<Stml>),
    FunctionDecl(
        Rc<Token>,
        Rc<Token>,
        Vec<Expr>,
        Vec<(Expr, Expr)>,
        Option<(Rc<Token>, Box<Expr>)>,
        Box<Stml>,
    ),
    VarDecl(Rc<Token>, Vec<(Expr, Option<Expr>)>),
    Return(Rc<Token>, Option<Expr>),
    Throw(Rc<Token>, Option<Expr>),
    TryCatch(Rc<Token>, Box<Stml>, Rc<Token>, Box<Stml>),
    IfElse(
        Rc<Token>,
        Expr,
        Box<Stml>,
        Vec<(Rc<Token>, Expr, Stml)>,
        Option<(Rc<Token>, Box<Stml>)>,
    ),
    While(Rc<Token>, Expr, Box<Stml>),
    Loop(Rc<Token>, Box<Stml>),
    Break(Rc<Token>),
    Continue(Rc<Token>),
    Import(Rc<Token>, Expr, Rc<Token>),
    Export(Rc<Token>, Box<Stml>),
    ForIn(Rc<Token>, Expr, Expr, Box<Stml>),
    Expr(Expr),
}

impl Stml {
    pub fn token(&self) -> Rc<Token> {
        match self {
            Self::Block(token, ..)
            | Self::FunctionDecl(token, ..)
            | Self::VarDecl(token, ..)
            | Self::Return(token, ..)
            | Self::Throw(token, ..)
            | Self::TryCatch(token, ..)
            | Self::IfElse(token, ..)
            | Self::While(token, ..)
            | Self::Loop(token, ..)
            | Self::Break(token)
            | Self::Continue(token)
            | Self::Import(token, ..)
            | Self::Export(token, ..)
            | Self::ForIn(token, ..) => Rc::clone(token),
            Self::Expr(expr) => expr.token(),
        }
    }
}
