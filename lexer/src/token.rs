extern crate variant_count;

use colored::Colorize;
use std::{cmp::PartialEq, convert::Into, fmt, path::PathBuf, rc::Rc, string};
use variant_count::VariantCount;

#[derive(Debug, Clone, Copy, PartialEq, VariantCount)]
pub enum TokenType {
    OParen,         // 0
    CParen,         // 1
    OBrace,         // 2
    CBrace,         // 3
    OBracket,       // 4
    CBracket,       // 5
    Period,         // 6
    TPeriod,        // 7
    Plus,           // 8
    Minus,          // 9
    Star,           // 10
    Slash,          // 11
    Percent,        // 12
    Comma,          // 13
    QuestionMark,   // 14
    Colon,          // 15
    Equal,          // 16
    PlusEqual,      // 17
    MinusEqual,     // 18
    StarEqual,      // 19
    SlashEqual,     // 20
    PercentEqual,   // 21
    DPlus,          // 22
    DMinus,         // 23
    DEqual,         // 24
    Bang,           // 25
    BangEqual,      // 26
    Greater,        // 27
    GreaterEqual,   // 28
    Less,           // 29
    LessEqual,      // 30
    And,            // 31
    Or,             // 32
    String,         // 33
    UnTermedString, // 34
    Comment,        // 35
    Identifier,     // 36
    If,             // 37
    ElseIf,         // 38
    Else,           // 39
    Function,       // 40
    Var,            // 41
    Loop,           // 42
    While,          // 43
    Break,          // 44
    Continue,       // 45
    Return,         // 46
    Throw,          // 47
    Try,            // 48
    Catch,          // 49
    Nil,            // 50
    True,           // 51
    False,          // 52
    Number,         // 53
    Import,         // 54
    From,           // 55
    Export,         // 56
    Pipe,           // 57
    For,            // 58
    In,             // 59
    Unknown,        // 60
    NewLine,        // 61
    EOF,            // 62
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
            Self::DPlus => "++",
            Self::DMinus => "--",

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
            Self::UnTermedString => "نص غير مغلق",
            Self::Comment => "تعليق",

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
        self.source.get(self.start..self.start + self.length).unwrap()
    }

    pub fn pos(&self) -> (usize, usize) {
        let mut offset = 0;
        let mut line = 1;
        let mut col = 1;
        for c in self.source.chars() {
            if offset == self.start {
                break;
            } else if c == '\n' {
                offset += 1;
                line += 1;
                col = 1;
            } else {
                offset += 1;
                col += 1;
            }
        }
        (line, col)
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
        let (line, col) = self.pos();
        f.debug_struct("Token")
            .field("type", &self.typ)
            .field("lexeme", &self.lexeme())
            .field("path", &self.path)
            .field("line", &line)
            .field("col", &col)
            .finish()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.pos();
        let indent = format!("{line}").len();
        match &self.path {
            Some(path) => writeln!(
                f,
                "{}",
                format!("{}--> {}", " ".repeat(indent), path.display()).bright_cyan()
            )?,
            None => {}
        };
        writeln!(f, "{}", format!("{} | ", " ".repeat(indent)).bright_cyan())?;
        write!(f, "{}", format!("{line} | ").bright_cyan())?;
        let mut line = self
            .source
            .lines()
            .nth(line - 1)
            .unwrap_or("") // ? if it's an ending empty line
            .chars()
            .collect::<Vec<char>>();
        let start = line.drain(0..col - 1).collect::<string::String>();
        let lexeme = line
            .drain(0..self.lexeme().chars().count())
            .collect::<string::String>();
        let end = line.drain(..).collect::<string::String>();
        writeln!(f, "{start}{}{end}", lexeme.underline().bold())?;
        write!(f, "{}", format!("{} | ", " ".repeat(indent)).bright_cyan())
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
