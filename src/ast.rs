use super::token::Token;
use std::{fmt, rc::Rc};

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

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub enum Stml<'a> {
    Block(Vec<Stml<'a>>),
    Function(Rc<Token<'a>>, Vec<Rc<Token<'a>>>, Box<Stml<'a>>),
    Return(Option<Box<Expr<'a>>>),
    Throw(Rc<Token<'a>>, Option<Box<Expr<'a>>>), //? We'll need it's token
    TryCatch(Box<Stml<'a>>, Rc<Token<'a>>, Box<Stml<'a>>),
    IfElse(Box<Expr<'a>>, Box<Stml<'a>>, Option<Box<Stml<'a>>>),
    While(Box<Expr<'a>>, Box<Stml<'a>>),
    Loop(Box<Stml<'a>>),
    Break,
    Continue,
    Expr(Expr<'a>),
}

impl<'a> Stml<'a> {
    fn to_string(&self) -> String {
        match self {
            Stml::Expr(expr) => return format!("{}\n", expr),
            Stml::Return(expr) => match expr {
                Some(expr) => return format!("أرجع {}\n", expr),
                None => return "أرجع عدم\n".to_string(),
            },
            Stml::Throw(_, expr) => match expr {
                Some(expr) => return format!("ألقي {}\n", expr),
                None => return "ألقي عدم\n".to_string(),
            },
            Stml::Function(name, params, body) => {
                let mut buffer = String::new();

                buffer += &format!(
                    "<دالة {} ({})>\n",
                    name.get_lexeme(),
                    params
                        .iter()
                        .map(|p| p.get_lexeme())
                        .collect::<Vec<_>>()
                        .join("، "),
                );
                buffer += &body.to_string();
                buffer += "<أنهي>\n";

                return buffer;
            }
            Stml::Block(stmls) => {
                let mut buffer = String::new();
                for stml in stmls {
                    buffer += &stml.to_string();
                }
                return buffer;
            }
            Stml::IfElse(expr, then_branch, else_branch) => {
                let mut buffer = String::new();
                buffer += &format!("<إن {}>\n", expr);
                buffer += &then_branch.to_string();
                match else_branch {
                    Some(else_branch) => {
                        buffer += "<إلا>\n";
                        buffer += &else_branch.to_string();
                    }
                    None => {}
                }
                buffer += "<أنهي>\n";
                return buffer;
            }
            Stml::While(expr, body) => {
                let mut buffer = String::new();
                buffer += &format!("<بينما {}>\n", expr);
                buffer += &body.to_string();
                buffer += "<أنهي>\n";
                return buffer;
            }
            Stml::Loop(body) => {
                let mut buffer = String::new();
                buffer += &format!("<كرر>\n");
                buffer += &body.to_string();
                buffer += "<أنهي>\n";
                return buffer;
            }
            Stml::TryCatch(try_branch, catch_token, catch_branch) => {
                let mut buffer = String::new();
                buffer += &format!("<حاول>\n");
                buffer += &try_branch.to_string();
                buffer += &format!("<ألقي {}>\n", catch_token.get_lexeme());
                buffer += &catch_branch.to_string();
                buffer += "<أنهي>\n";
                return buffer;
            }
            Stml::Break => return "قف\n".to_string(),
            Stml::Continue => return "أكمل\n".to_string(),
        }
    }
}

impl fmt::Display for Stml<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
