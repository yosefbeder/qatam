use colored::Colorize;
use lexer::token::{Token, TokenType};
use std::{fmt, rc::Rc};

#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectedInstead(Vec<TokenType>, Rc<Token>),
    ExpectedExpr(Rc<Token>),
    InvalidRhs(Rc<Token>),
    ExpectedOptional(Rc<Token>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "خطأ تحليلي: ".bright_red())?;
        match self {
            Self::ExpectedInstead(expected, token) => {
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
            Self::ExpectedExpr(token) => {
                let got: &str = token.typ().to_owned().into();
                write!(f, "توقعت عبارة ولكن حصلت على \"{got}\"\n{token}")
            }
            Self::InvalidRhs(token) => {
                write!(f, "الجانب الأيمن لعلامة التساوي غير صحيح\n{token}")
            }
            Self::ExpectedOptional(token) => {
                write!(f, "لا يمكن وضع مدخل إجباري بعد مدخل إختياري\n{token}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Lexical(Rc<Token>),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexical(token) => {
                let typ: &str = token.typ().to_owned().into();
                write!(f, "{}{typ}\n{token}", "خطأ كلمي: ".bright_red())
            }
            Self::Parse(err) => write!(f, "{err}"),
        }
    }
}
