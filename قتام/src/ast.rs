use super::token::Token;
use std::{fmt, rc::Rc};

//? here all of the tokens are wrapped inside an rc smart pointer because I'm going to store them also them in the bytecode
pub enum Literal {
    Number(Rc<Token>),
    String(Rc<Token>),
    Bool(Rc<Token>),
    Nil(Rc<Token>),
    List(Vec<Expr>),
    Object(Vec<(Rc<Token>, Expr)>),
}

pub enum Expr {
    Variable(Rc<Token>),
    Literal(Literal),
    Unary(Rc<Token>, Box<Expr>),
    Binary(Rc<Token>, Box<Expr>, Box<Expr>),
    Call(Rc<Token>, Box<Expr>, Vec<Expr>),
    Get(Rc<Token>, Box<Expr>, Box<Expr>),
    Set(Rc<Token>, Box<Expr>, Box<Expr>, Box<Expr>),
}

impl fmt::Debug for Expr {
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

pub enum Stml {
    Block(Vec<Stml>),
    FunctionDecl(Rc<Token>, Vec<Rc<Token>>, Box<Stml>),
    VarDecl(Rc<Token>, Option<Expr>),
    Return(Rc<Token>, Option<Expr>),
    Throw(Rc<Token>, Option<Expr>),
    TryCatch(Box<Stml>, Rc<Token>, Box<Stml>),
    IfElse(Expr, Box<Stml>, Option<Box<Stml>>),
    While(Expr, Box<Stml>),
    Loop(Box<Stml>),
    Break(Rc<Token>),
    Continue(Rc<Token>),
    Import(Rc<Token>, Rc<Token>),
    Export(Rc<Token>, Box<Stml>),
    Expr(Expr),
}

impl Stml {
    fn as_block(&self) -> &Vec<Stml> {
        match self {
            Self::Block(stmts) => stmts,
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for Stml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_as_block(buffer: &mut String, block: &Stml) {
            for stml in block.as_block() {
                *buffer += format!("{stml:?}").as_str();
            }
        }

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
                    fmt_as_block(&mut buffer, body);
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
                    fmt_as_block(&mut buffer, then_branch);

                    match else_branch {
                        Some(else_branch) => {
                            buffer += "<إلا>\n";
                            fmt_as_block(&mut buffer, else_branch);
                        }
                        None => {}
                    }
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::While(expr, body) => {
                    let mut buffer = String::new();
                    buffer += &format!("<بينما {:?}>\n", expr);
                    fmt_as_block(&mut buffer, body);
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::Loop(body) => {
                    let mut buffer = String::new();
                    buffer += &format!("<كرر>\n");
                    fmt_as_block(&mut buffer, body);
                    buffer += "<أنهي>\n";
                    buffer
                }
                Stml::Break(_) => "<قف>\n".to_string(),
                Stml::Continue(_) => "<أكمل>\n".to_string(),
                Stml::Import(name, path) => format!(
                    "<Export {} من {}>\n",
                    name.lexeme.clone(),
                    path.lexeme.clone()
                ),
                Stml::Export(_, stml) => {
                    let mut buffer = String::new();
                    buffer += &format!("<تصدير\n{stml:?}>\n");
                    buffer
                }
                Stml::TryCatch(try_block, error, catch_block) => {
                    let mut buffer = String::new();
                    buffer += "<جرب>\n";
                    fmt_as_block(&mut buffer, try_block);
                    buffer += format!("<أمسك {}>\n", error.lexeme.clone()).as_str();
                    fmt_as_block(&mut buffer, catch_block);
                    buffer += "<أنهي>\n";
                    buffer
                }
            }
        )
    }
}
