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

pub const BOUNDARIES: [TokenType; 14] = [
    TokenType::Function,
    TokenType::Var,
    TokenType::While,
    TokenType::Loop,
    TokenType::If,
    TokenType::Try,
    TokenType::OBrace,
    TokenType::Break,
    TokenType::Continue,
    TokenType::Return,
    TokenType::Throw,
    TokenType::Import,
    TokenType::Export,
    TokenType::For,
];

pub const BINARY_SET: [TokenType; 6] = [
    TokenType::Equal,
    TokenType::PlusEqual,
    TokenType::MinusEqual,
    TokenType::StarEqual,
    TokenType::SlashEqual,
    TokenType::PercentEqual,
];

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

#[derive(Debug, Clone)]
pub struct TokenPos {
    path: Option<PathBuf>,
    line: usize,
    col: usize,
}

impl TokenPos {
    fn new(path: &Option<PathBuf>, line: usize, col: usize) -> Self {
        Self {
            path: path.clone(),
            line,
            col,
        }
    }
}

impl fmt::Display for TokenPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { path, line, col } = self;

        if let Some(path) = path {
            write!(f, "[الملف: {}، ", path.display())?;
        } else {
            write!(f, "[")?;
        }

        write!(f, "السطر: {line}، العمود: {col}]")
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
        let TokenPos { path, line, col } = self.get_pos();
        let padding = " ".repeat(4);
        writeln!(f, "{{")?;
        writeln!(f, "{padding}type: {:?},", self.typ)?;
        writeln!(f, "{padding}lexeme: {:?},", self.lexeme)?;
        writeln!(f, "{padding}path: {path:?},")?;
        writeln!(f, "{padding}line: {line},")?;
        writeln!(f, "{padding}col: {col},")?;
        write!(f, "}}")
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

    pub fn new_empty() -> Self {
        Self {
            typ: TokenType::Unknown,
            source: Rc::new("".to_string()),
            path: None,
            lexeme: "".to_string(),
            start: 0,
        }
    }

    pub fn get_pos(&self) -> TokenPos {
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
        TokenPos::new(&self.path, line, col)
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
