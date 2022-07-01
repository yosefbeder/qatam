use std::{cmp::PartialEq, convert::From, fmt, path::PathBuf, rc::Rc};

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
    InvalidNumber,
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
        match typ {
            TokenType::NewLine => 0,
            TokenType::OParen => 1,
            TokenType::CParen => 2,
            TokenType::OBrace => 3,
            TokenType::CBrace => 4,
            TokenType::OBracket => 5,
            TokenType::CBracket => 6,
            TokenType::Period => 7,
            TokenType::Plus => 8,
            TokenType::Minus => 9,
            TokenType::Star => 10,
            TokenType::Slash => 11,
            TokenType::Percent => 12,
            TokenType::Comma => 13,
            TokenType::QuestionMark => 14,
            TokenType::Colon => 15,

            TokenType::Equal => 16,
            TokenType::PlusEqual => 17,
            TokenType::MinusEqual => 18,
            TokenType::StarEqual => 19,
            TokenType::SlashEqual => 20,
            TokenType::PercentEqual => 21,
            TokenType::DPlus => 22,
            TokenType::DMinus => 23,

            TokenType::DEqual => 24,
            TokenType::Bang => 25,
            TokenType::BangEqual => 26,
            TokenType::Greater => 27,
            TokenType::GreaterEqual => 28,
            TokenType::Less => 29,
            TokenType::LessEqual => 30,
            TokenType::And => 31,
            TokenType::Or => 32,

            TokenType::String => 33,
            TokenType::UnTermedString => 34,
            TokenType::Comment => 35,

            TokenType::Identifier => 36,
            TokenType::If => 37,
            TokenType::ElseIf => 38,
            TokenType::Else => 39,
            TokenType::Function => 40,
            TokenType::Var => 41,
            TokenType::Loop => 42,
            TokenType::While => 43,
            TokenType::Break => 44,
            TokenType::Continue => 45,
            TokenType::Return => 46,
            TokenType::Throw => 47,
            TokenType::Try => 48,
            TokenType::Catch => 49,
            TokenType::Nil => 50,
            TokenType::True => 51,
            TokenType::False => 52,
            TokenType::Number => 53,
            TokenType::InvalidNumber => 54,

            TokenType::Import => 55,
            TokenType::From => 56,
            TokenType::Export => 57,
            TokenType::Pipe => 58,
            TokenType::For => 59,
            TokenType::In => 60,
            TokenType::Unknown => 61,
            TokenType::EOF => 62,
        }
    }
}

pub const NUMBER: usize = 63;

#[derive(Clone)]
pub struct Token {
    pub typ: TokenType,
    source: Rc<String>,
    path: Option<PathBuf>,
    pub lexeme: String,
    start: usize,
    length: usize,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.get_pos();
        write!(
            f,
            "{{\n{3}type: {0:?},\n{3}lexeme: {1},\n{3}path:{2:?},\n{3}line: {line},\n{3}column: {col},\n}}",
            self.typ,
            self.lexeme,
            self.path,
            " ".repeat(4),
        )
    }
}

impl Token {
    pub fn new(
        typ: TokenType,
        source: Rc<String>,
        path: &Option<PathBuf>,
        lexeme: String,
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
            lexeme,
            start,
            length,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            typ: TokenType::Unknown,
            source: Rc::new("".to_string()),
            path: None,
            lexeme: "".to_string(),
            start: 0,
            length: 0,
        }
    }

    pub fn get_pos(&self) -> (usize, usize) {
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
            | TokenType::InvalidNumber
            | TokenType::Unknown => self.typ == other.typ && self.lexeme == other.lexeme,
            _ => self.typ == other.typ,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.get_pos();
        let content = self.source.lines().nth(line - 1).unwrap();
        let padding = " ".repeat(format!("{line}").len() + 1);

        let mut buffer = String::new();
        if self.path.is_some() {
            buffer += padding.as_str();
            buffer += "|\n";
            buffer += format!("{line} | {content}\n").as_str();
            buffer += padding.as_str();
            buffer += "| ";
        } else {
            buffer += format!("{content}\n").as_str();
        }
        for _ in 0..col - 1 {
            buffer += " ";
        }
        for _ in 0..self.length {
            buffer += "~";
        }
        if let Some(path) = &self.path {
            buffer += format!("\n{}", path.display()).as_str();
        }
        write!(f, "{}", buffer,)
    }
}
