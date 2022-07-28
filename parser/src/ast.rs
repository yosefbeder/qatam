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
    Lambda(Rc<Token>, Vec<(Expr, Option<Expr>)>, Box<Stml>),
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
            | Self::Literal(Literal::List(token, _))
            | Self::Literal(Literal::Object(token, _))
            | Self::Literal(Literal::Lambda(token, _, _)) => Rc::clone(token),
            Self::Unary(oper, _)
            | Self::Binary(oper, _, _)
            | Self::Call(oper, _, _)
            | Self::Member(oper, _, _) => Rc::clone(oper),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stml {
    Block(Rc<Token>, Vec<Stml>),
    FunctionDecl(Rc<Token>, Rc<Token>, Vec<(Expr, Option<Expr>)>, Box<Stml>),
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
            Self::Block(token, _)
            | Self::FunctionDecl(token, _, _, _)
            | Self::VarDecl(token, _)
            | Self::Return(token, _)
            | Self::Throw(token, _)
            | Self::TryCatch(token, _, _, _)
            | Self::IfElse(token, _, _, _, _)
            | Self::While(token, _, _)
            | Self::Loop(token, _)
            | Self::Break(token)
            | Self::Continue(token)
            | Self::Import(token, _, _)
            | Self::Export(token, _)
            | Self::ForIn(token, _, _, _) => Rc::clone(token),
            Self::Expr(expr) => expr.token(),
        }
    }
}
