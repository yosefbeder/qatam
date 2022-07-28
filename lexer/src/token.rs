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
    TPeriod,
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
            TPeriod => 8,
            Plus => 9,
            Minus => 10,
            Star => 11,
            Slash => 12,
            Percent => 13,
            Comma => 14,
            QuestionMark => 15,
            Colon => 16,

            Equal => 17,
            PlusEqual => 18,
            MinusEqual => 19,
            StarEqual => 20,
            SlashEqual => 21,
            PercentEqual => 22,
            DPlus => 23,
            DMinus => 24,

            DEqual => 25,
            Bang => 26,
            BangEqual => 27,
            Greater => 28,
            GreaterEqual => 29,
            Less => 30,
            LessEqual => 31,
            And => 32,
            Or => 33,

            String => 34,
            UnTermedString => 35,
            Comment => 36,

            Identifier => 37,
            If => 38,
            ElseIf => 39,
            Else => 40,
            Function => 41,
            Var => 42,
            Loop => 43,
            While => 44,
            Break => 45,
            Continue => 46,
            Return => 47,
            Throw => 48,
            Try => 49,
            Catch => 50,
            Nil => 51,
            True => 52,
            False => 53,
            Number => 54,

            Import => 55,
            From => 56,
            Export => 57,
            Pipe => 58,
            For => 59,
            In => 60,
            Unknown => 61,
            EOF => 62,
        }
    }
}

pub const NUMBER: usize = 63;

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
            TPeriod => "...",
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
