use super::token::Token;
use std::rc::Rc;

//? here all of the tokens are wrapped inside an rc smart pointer because I'm going to store them also them in the bytecode
#[derive(Debug)]
pub enum Literal<'a> {
    Number(Rc<Token<'a>>),
    String(Rc<Token<'a>>),
    Bool(Rc<Token<'a>>),
    Nil(Rc<Token<'a>>),
    List(Vec<Expr<'a>>),
    Object(Vec<(Rc<Token<'a>>, Expr<'a>)>),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Variable(Rc<Token<'a>>),
    Literal(Literal<'a>),
    Unary(Rc<Token<'a>>, Box<Expr<'a>>),
    Binary(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    Call(Rc<Token<'a>>, Box<Expr<'a>>, Vec<Expr<'a>>),
    Get(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    Set(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
}

impl<'a> Expr<'a> {
    pub fn to_string(&self) -> String {
        match self {
            Self::Variable(token) => token.get_lexeme(),
            Self::Literal(literal) => match literal {
                Literal::Number(token)
                | Literal::String(token)
                | Literal::Bool(token)
                | Literal::Nil(token) => token.get_lexeme(),
                Literal::List(_) => "<قائمة>".to_string(),
                Literal::Object(_) => "<كائن>".to_string(),
            },
            Self::Unary(token, expr) => format!("({} {})", token.get_lexeme(), expr.to_string()),
            Self::Binary(token, left, right) => format!(
                "({} {} {})",
                token.get_lexeme(),
                left.to_string(),
                right.to_string()
            ),
            Self::Call(_, callee, args) => {
                format!(
                    "(استدعي {} [{}])",
                    callee.to_string(),
                    args.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Self::Get(_, expr, key) => {
                format!("(أحضر {} {})", expr.to_string(), key.to_string())
            }
            Self::Set(_, expr, key, right) => {
                format!(
                    "(إجعل {} {} {})",
                    expr.to_string(),
                    key.to_string(),
                    right.to_string()
                )
            }
        }
    }
}
