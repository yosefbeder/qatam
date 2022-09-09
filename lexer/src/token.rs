extern crate variant_count;

use colored::Colorize;
use std::{cmp::PartialEq, convert::Into, fmt, path::PathBuf, rc::Rc, string};
use variant_count::VariantCount;

use super::is_newline;

#[derive(Debug, Clone, Copy, PartialEq, VariantCount)]
pub enum TokenType {
    OParen,                   // 0
    CParen,                   // 1
    OBrace,                   // 2
    CBrace,                   // 3
    OBracket,                 // 4
    CBracket,                 // 5
    Period,                   // 6
    TPeriod,                  // 7
    Plus,                     // 8
    Minus,                    // 9
    Star,                     // 10
    Slash,                    // 11
    Percent,                  // 12
    Comma,                    // 13
    QuestionMark,             // 14
    Colon,                    // 15
    Equal,                    // 16
    PlusEqual,                // 17
    MinusEqual,               // 18
    StarEqual,                // 19
    SlashEqual,               // 20
    PercentEqual,             // 21
    DEqual,                   // 22
    Bang,                     // 23
    BangEqual,                // 24
    Greater,                  // 25
    GreaterEqual,             // 26
    Less,                     // 27
    LessEqual,                // 28
    And,                      // 29
    Or,                       // 30
    String,                   // 31
    UnterminatedString,       // 32
    InlineComment,            // 33
    BlockComment,             // 34
    UnterminatedBlockComment, // 35
    Identifier,               // 36
    If,                       // 37
    ElseIf,                   // 38
    Else,                     // 39
    Function,                 // 38
    Var,                      // 39
    Loop,                     // 40
    While,                    // 41
    Break,                    // 42
    Continue,                 // 43
    Return,                   // 44
    Throw,                    // 45
    Try,                      // 46
    Catch,                    // 47
    Nil,                      // 48
    True,                     // 49
    False,                    // 50
    Number,                   // 51
    Import,                   // 52
    From,                     // 53
    Export,                   // 54
    Pipe,                     // 55
    For,                      // 56
    In,                       // 57
    Unknown,                  // 58
    NewLine,                  // 59
    EOF,                      // 60
}

impl Into<&'static str> for TokenType {
    fn into(self) -> &'static str {
        match self {
            Self::NewLine => "سطر جديد",
            Self::OParen => "(",
            Self::CParen => ")",
            Self::OBrace => "{",
            Self::CBrace => "}",
            Self::OBracket => "]",
            Self::CBracket => "[",
            Self::Period => ".",
            Self::TPeriod => "...",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Comma => "،",
            Self::QuestionMark => "؟",
            Self::Colon => ":",

            Self::Equal => "=",
            Self::PlusEqual => "+=",
            Self::MinusEqual => "-=",
            Self::StarEqual => "*=",
            Self::SlashEqual => "/=",
            Self::PercentEqual => "%=",

            Self::DEqual => "==",
            Self::Bang => "!",
            Self::BangEqual => "!=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::And => "&&",
            Self::Or => "||",

            Self::String => "نص",
            Self::UnterminatedString => "نص غير مغلق",
            Self::InlineComment => "تعليق سطري",
            Self::BlockComment => "تعليق",
            Self::UnterminatedBlockComment => "تعليق غير مغلق",

            Self::Identifier => "كلمة",
            Self::If => "إن",
            Self::ElseIf => "وإن",
            Self::Else => "إلا",
            Self::Function => "دالة",
            Self::Var => "متغير",
            Self::Loop => "كرر",
            Self::While => "طالما",
            Self::Break => "إكسر",
            Self::Continue => "واصل",
            Self::Return => "أرجع",
            Self::Throw => "ألقي",
            Self::Try => "حاول",
            Self::Catch => "أمسك",
            Self::Nil => "عدم",
            Self::True => "صحيح",
            Self::False => "خطأ",
            Self::Number => "رقم",

            Self::Import => "استورد",
            Self::From => "من",
            Self::Export => "صدّر",
            Self::Pipe => "|",
            Self::For => "لكل",
            Self::In => "في",
            Self::Unknown => "حرف غير معروف",
            Self::EOF => "النهاية",
        }
    }
}

impl Token {
    pub fn new(
        typ: TokenType,
        source: Rc<string::String>,
        path: Option<&PathBuf>,
        start: usize,
        length: usize,
    ) -> Self {
        Self {
            typ,
            source,
            path: match path {
                Some(path) => Some(path.clone()),
                None => None,
            },
            start,
            length,
        }
    }

    pub fn typ(&self) -> TokenType {
        self.typ
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn lexeme(&self) -> &str {
        self.source
            .get(self.start..self.start + self.length)
            .unwrap()
    }

    pub fn line(&self) -> usize {
        let mut line = 1;
        for (offset, c) in self.source.char_indices() {
            if is_newline(c) {
                line += 1;
            }
            if offset == self.start {
                break;
            }
        }
        line
    }
}

#[derive(Clone)]
pub struct Token {
    typ: TokenType,
    source: Rc<String>,
    path: Option<PathBuf>,
    start: usize,
    length: usize,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.typ)
            .field("lexeme", &self.lexeme())
            .field("path", &self.path)
            .finish()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut char_indices = self.source.char_indices().peekable();
        let mut line_idx = 0;
        let mut line_start_offset = 0;
        while let Some((offset, c)) = char_indices.next() {
            if is_newline(c) {
                line_idx += 1;
                line_start_offset = offset;
            }
            if offset == self.start {
                while let Some((offset, _)) = char_indices.peek() {
                    if *offset == self.start + self.length {
                        break;
                    } else {
                        char_indices.next();
                    }
                }
                break;
            }
        }
        let line = line_idx + 1;
        let indent = (line_idx + 1).to_string().len();
        if let Some(path) = self.path.as_ref() {
            writeln!(
                f,
                "{:indent$}{} {}",
                "",
                "-->".bright_cyan(),
                path.display().to_string().bright_cyan()
            )?
        }
        writeln!(f, "{:indent$} {}", "", "|".bright_cyan())?;
        write!(
            f,
            "{} {} ",
            line.to_string().bright_cyan(),
            "|".bright_cyan()
        )?;
        write!(
            f,
            "{}{}",
            self.source.get(line_start_offset..self.start).unwrap(),
            self.lexeme().underline().bold()
        )?;
        while let Some((_, c)) = char_indices.next() {
            if is_newline(c) {
                break;
            } else {
                write!(f, "{c}")?
            }
        }
        write!(f, "\n")?;
        writeln!(f, "{:indent$} {}", "", "|".bright_cyan())?;
        Ok(())
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            typ: TokenType::Unknown,
            source: Rc::new("".to_string()),
            path: None,
            start: 0,
            length: 0,
        }
    }
}

pub trait TokenInside {
    fn token(&self) -> Rc<Token>;
}

pub const ERROR_TOKENS: [TokenType; 3] = [
    TokenType::Unknown,
    TokenType::UnterminatedString,
    TokenType::UnterminatedBlockComment,
];

pub const BINARY_SET: [TokenType; 6] = [
    TokenType::Equal,
    TokenType::PlusEqual,
    TokenType::MinusEqual,
    TokenType::StarEqual,
    TokenType::SlashEqual,
    TokenType::PercentEqual,
];
