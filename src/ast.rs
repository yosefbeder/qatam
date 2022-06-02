use super::token::Token;
use std::{fmt, rc::Rc};

//? here all of the tokens are wrapped inside an rc smart pointer because I'm going to store them also them in the bytecode
pub enum Literal<'a> {
    Number(Rc<Token<'a>>),
    String(Rc<Token<'a>>),
    Bool(Rc<Token<'a>>),
    Nil(Rc<Token<'a>>),
    List(Vec<Expr<'a>>),
    Object(Vec<(Rc<Token<'a>>, Expr<'a>)>),
}

pub enum Expr<'a> {
    Variable(Rc<Token<'a>>),
    Literal(Literal<'a>),
    Unary(Rc<Token<'a>>, Box<Expr<'a>>),
    Binary(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    Call(Rc<Token<'a>>, Box<Expr<'a>>, Vec<Expr<'a>>),
    Get(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    Set(Rc<Token<'a>>, Box<Expr<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
}

impl fmt::Debug for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Variable(token) => token.lexeme.clone(),
                Self::Literal(literal) => match literal {
                    Literal::Number(token)
                    | Literal::String(token)
                    | Literal::Bool(token)
                    | Literal::Nil(token) => token.lexeme.clone(),
                    Literal::List(_) => "<قائمة>".to_string(),
                    Literal::Object(_) => "<كائن>".to_string(),
                },
                Self::Unary(token, expr) => format!("({} {:?})", token.lexeme.clone(), expr),
                Self::Binary(token, left, right) =>
                    format!("({} {:?} {:?})", token.lexeme.clone(), left, right),
                Self::Call(_, callee, args) => {
                    format!(
                        "(استدعي {:?} [{}])",
                        callee,
                        args.iter()
                            .map(|e| format!("{:?}", e))
                            .collect::<Vec<String>>()
                            .join(" ")
                    )
                }
                Self::Get(_, expr, key) => {
                    format!("(أحضر {:?} {:?})", expr, key)
                }
                Self::Set(_, expr, key, right) => {
                    format!("(إجعل {:?} {:?} {:?})", expr, key, right)
                }
            }
        )
    }
}

pub enum Stml<'a> {
    Block(Vec<Stml<'a>>),
    FunctionDecl(Rc<Token<'a>>, Vec<Rc<Token<'a>>>, Box<Stml<'a>>),
    VarDecl(Rc<Token<'a>>, Option<Expr<'a>>),
    Return(Rc<Token<'a>>, Option<Expr<'a>>),
    Throw(Rc<Token<'a>>, Option<Expr<'a>>), //? We'll need it's token
    TryCatch(Box<Stml<'a>>, Rc<Token<'a>>, Box<Stml<'a>>),
    IfElse(Expr<'a>, Box<Stml<'a>>, Option<Box<Stml<'a>>>),
    While(Expr<'a>, Box<Stml<'a>>),
    Loop(Box<Stml<'a>>),
    Break(Rc<Token<'a>>),
    Continue(Rc<Token<'a>>),
    Expr(Expr<'a>),
}

impl fmt::Debug for Stml<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Stml::Expr(expr) => format!("{:?}\n", expr),
                Stml::Return(_, expr) => match expr {
                    Some(expr) => format!("<أرجع {:?}>\n", expr),
                    None => "<أرجع عدم>\n".to_string(),
                },
                Stml::Throw(_, expr) => match expr {
                    Some(expr) => format!("<ألقي {:?}>\n", expr),
                    None => "<ألقي عدم>\n".to_string(),
                },
                Stml::FunctionDecl(name, params, body) => {
                    let mut buffer = String::new();

                    buffer += &format!(
                        "<دالة {} ({})>\n",
                        name.lexeme.clone(),
                        params
                            .iter()
                            .map(|p| p.lexeme.clone())
                            .collect::<Vec<_>>()
                            .join("، "),
                    );
                    let stmls = match &**body {
                        Stml::Block(stmls) => stmls,
                        _ => unreachable!(),
                    };
                    for stml in stmls {
                        buffer += format!("{:?}", stml).as_str();
                    }
                    buffer += "<أنهي>\n";

                    buffer
                }
                Stml::VarDecl(name, initializer) => format!(
                    "<تعريف {} {}>\n",
                    name.lexeme.clone(),
                    match initializer {
                        Some(expr) => format!("{:?}", expr),
                        None => "عدم".to_string(),
                    }
                ),
                Stml::Block(stmls) => {
                    let mut buffer = String::new();
                    buffer += "<مجموعة>\n";
                    for stml in stmls {
                        buffer += format!("{:?}", stml).as_str();
                    }
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::IfElse(expr, then_branch, else_branch) => {
                    let mut buffer = String::new();
                    buffer += format!("<إن {:?}>\n", expr).as_str();
                    buffer += format!("{:?}", then_branch).as_str();
                    match else_branch {
                        Some(else_branch) => {
                            buffer += "<إلا>\n";
                            buffer += format!("{:?}", else_branch).as_str();
                        }
                        None => {}
                    }
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::While(expr, body) => {
                    let mut buffer = String::new();
                    buffer += &format!("<بينما {:?}>\n", expr);
                    let stmls = match &**body {
                        Stml::Block(stmls) => stmls,
                        _ => unreachable!(),
                    };
                    for stml in stmls {
                        buffer += format!("{:?}", stml).as_str();
                    }
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::Loop(body) => {
                    let mut buffer = String::new();
                    buffer += &format!("<كرر>\n");
                    let stmls = match &**body {
                        Stml::Block(stmls) => stmls,
                        _ => unreachable!(),
                    };
                    for stml in stmls {
                        buffer += format!("{:?}", stml).as_str();
                    }
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::Break(_) => "<قف>\n".to_string(),
                Stml::Continue(_) => "<أكمل>\n".to_string(),
                Stml::TryCatch(_, _, _) => unimplemented!(),
            }
        )
    }
}
