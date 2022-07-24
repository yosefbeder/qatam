use colored::Colorize;
use std::{cmp::PartialEq, convert::From, convert::Into, fmt, path::PathBuf, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    OParen,
    CParen,
    OBrace,
    CBrace,
    OBracket,
    CBracket,
    Period,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Comma,
    QuestionMark,
    Colon,

    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    DPlus,
    DMinus,

    DEqual,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,

    String,
    UnTermedString,
    Comment,

    Identifier,
    If,
    ElseIf,
    Else,
    Function,
    Var,
    Loop,
    While,
    Break,
    Continue,
    Return,
    Throw,
    Try,
    Catch,
    Nil,
    True,
    False,
    Number,
    Import,
    From,
    Export,
    Pipe,
    For,
    In,

    Unknown,
    NewLine,
    EOF,
}

impl From<TokenType> for usize {
    fn from(typ: TokenType) -> usize {
        use TokenType::*;
        match typ {
            NewLine => 0,
            OParen => 1,
            CParen => 2,
            OBrace => 3,
            CBrace => 4,
            OBracket => 5,
            CBracket => 6,
            Period => 7,
            Plus => 8,
            Minus => 9,
            Star => 10,
            Slash => 11,
            Percent => 12,
            Comma => 13,
            QuestionMark => 14,
            Colon => 15,

            Equal => 16,
            PlusEqual => 17,
            MinusEqual => 18,
            StarEqual => 19,
            SlashEqual => 20,
            PercentEqual => 21,
            DPlus => 22,
            DMinus => 23,

            DEqual => 24,
            Bang => 25,
            BangEqual => 26,
            Greater => 27,
            GreaterEqual => 28,
            Less => 29,
            LessEqual => 30,
            And => 31,
            Or => 32,

            String => 33,
            UnTermedString => 34,
            Comment => 35,

            Identifier => 36,
            If => 37,
            ElseIf => 38,
            Else => 39,
            Function => 40,
            Var => 41,
            Loop => 42,
            While => 43,
            Break => 44,
            Continue => 45,
            Return => 46,
            Throw => 47,
            Try => 48,
            Catch => 49,
            Nil => 50,
            True => 51,
            False => 52,
            Number => 53,

            Import => 54,
            From => 55,
            Export => 56,
            Pipe => 57,
            For => 58,
            In => 59,
            Unknown => 60,
            EOF => 61,
        }
    }
}

pub const NUMBER: usize = 62;

impl Into<&'static str> for TokenType {
    fn into(self) -> &'static str {
        use TokenType::*;
        match self {
            NewLine => "سطر جديد",
            OParen => "(",
            CParen => ")",
            OBrace => "{",
            CBrace => "}",
            OBracket => "]",
            CBracket => "[",
            Period => ".",
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Percent => "%",
            Comma => "،",
            QuestionMark => "؟",
            Colon => ":",

            Equal => "=",
            PlusEqual => "+=",
            MinusEqual => "-=",
            StarEqual => "*=",
            SlashEqual => "/=",
            PercentEqual => "%=",
            DPlus => "++",
            DMinus => "--",

            DEqual => "==",
            Bang => "!",
            BangEqual => "!=",
            Greater => ">",
            GreaterEqual => ">=",
            Less => "<",
            LessEqual => "<=",
            And => "&&",
            Or => "||",

            String => "نص",
            UnTermedString => "نص غير مغلق",
            Comment => "تعليق",

            Identifier => "كلمة",
            If => "إن",
            ElseIf => "وإن",
            Else => "إلا",
            Function => "دالة",
            Var => "متغير",
            Loop => "كرر",
            While => "طالما",
            Break => "إكسر",
            Continue => "واصل",
            Return => "أرجع",
            Throw => "ألقي",
            Try => "حاول",
            Catch => "أمسك",
            Nil => "عدم",
            True => "صحيح",
            False => "خطأ",
            Number => "رقم",

            Import => "استورد",
            From => "من",
            Export => "صدّر",
            Pipe => "|",
            For => "لكل",
            In => "في",
            Unknown => "حرف غير معروف",
            EOF => "النهاية",
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub typ: TokenType,
    source: Rc<String>,
    path: Option<PathBuf>,
    pub lexeme: String,
    start: usize,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.pos();
        f.debug_struct("Token")
            .field("type", &self.typ)
            .field("lexeme", &self.lexeme)
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
            .unwrap()
            .chars()
            .collect::<Vec<char>>();
        let start = line.drain(0..col - 1).collect::<String>();
        let lexeme = line
            .drain(0..self.lexeme.chars().count())
            .collect::<String>();
        let end = line.drain(..).collect::<String>();
        writeln!(f, "{start}{}{end}", lexeme.underline().bold())?;
        write!(f, "{}", format!("{} | ", " ".repeat(indent)).bright_cyan())
    }
}

impl Token {
    pub fn new(
        typ: TokenType,
        source: Rc<String>,
        path: &Option<PathBuf>,
        lexeme: String,
        start: usize,
    ) -> Self {
        Self {
            typ,
            source,
            path: match path {
                Some(path) => Some(path.clone()),
                None => None,
            },
            lexeme,
            start,
        }
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

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self.typ {
            TokenType::Identifier
            | TokenType::String
            | TokenType::UnTermedString
            | TokenType::Number
            | TokenType::Unknown => self.typ == other.typ && self.lexeme == other.lexeme,
            _ => self.typ == other.typ,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            typ: TokenType::Unknown,
            source: Rc::new("".to_string()),
            path: None,
            lexeme: "".to_string(),
            start: 0,
        }
    }
}
