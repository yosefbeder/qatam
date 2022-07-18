use super::token::Token;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Rc<Token>),
    String(Rc<Token>),
    Bool(Rc<Token>),
    Nil(Rc<Token>),
    List(Vec<Expr>),
    Object(Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Variable(Rc<Token>),
    Literal(Literal),
    Unary(Rc<Token>, Box<Expr>),
    Binary(Rc<Token>, Box<Expr>, Box<Expr>),
    Call(Rc<Token>, Box<Expr>, Vec<Expr>),
    Member(Rc<Token>, Box<Expr>, Box<Expr>),
    Lambda(Rc<Token>, Vec<(Expr, Option<Expr>)>, Box<Stml>),
}

impl Expr {
    pub fn is_variable(&self) -> bool {
        match self {
            Self::Variable(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stml {
    Block(Vec<Stml>),
    FunctionDecl(Rc<Token>, Vec<(Expr, Option<Expr>)>, Box<Stml>),
    VarDecl(Rc<Token>, Vec<(Expr, Option<Expr>)>),
    Return(Rc<Token>, Option<Expr>),
    Throw(Rc<Token>, Option<Expr>),
    TryCatch(Box<Stml>, Rc<Token>, Box<Stml>),
    IfElse(Expr, Box<Stml>, Vec<(Expr, Stml)>, Option<Box<Stml>>),
    While(Expr, Box<Stml>),
    Loop(Box<Stml>),
    Break(Rc<Token>),
    Continue(Rc<Token>),
    Import(Rc<Token>, Expr, Rc<Token>),
    Export(Rc<Token>, Box<Stml>),
    ForIn(Rc<Token>, Expr, Expr, Box<Stml>),
    Expr(Expr),
}

impl Stml {
    pub fn as_block(&self) -> &Vec<Stml> {
        match self {
            Self::Block(decls) => decls,
            _ => unreachable!(),
        }
    }
}
