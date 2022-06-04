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

    Unknown,
    NewLine,
    EOF,
}

pub const BOUNDARIES: [TokenType; 13] = [
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
            TokenType::DEqual => 17,
            TokenType::Bang => 18,
            TokenType::BangEqual => 19,
            TokenType::Greater => 20,
            TokenType::GreaterEqual => 21,
            TokenType::Less => 22,
            TokenType::LessEqual => 23,
            TokenType::And => 24,
            TokenType::Or => 25,

            TokenType::String => 26,
            TokenType::UnTermedString => 27,
            TokenType::Comment => 28,

            TokenType::Identifier => 29,
            TokenType::If => 30,
            TokenType::ElseIf => 31,
            TokenType::Else => 32,
            TokenType::Function => 33,
            TokenType::Var => 34,
            TokenType::Loop => 35,
            TokenType::While => 36,
            TokenType::Break => 37,
            TokenType::Continue => 38,
            TokenType::Return => 39,
            TokenType::Throw => 40,
            TokenType::Try => 41,
            TokenType::Catch => 42,
            TokenType::Nil => 43,
            TokenType::True => 44,
            TokenType::False => 45,
            TokenType::Number => 46,
            TokenType::InvalidNumber => 47,

            TokenType::Import => 48,
            TokenType::From => 49,
            TokenType::Export => 50,
            TokenType::Unknown => 51,
            TokenType::EOF => 52,
        }
    }
}

pub const NUMBER: usize = 53;

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

    fn get_pos(&self) -> (usize, usize) {
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
        match &self.path {
            Some(path) => {
                buffer += format!("{}\n", path.display()).as_str();
                buffer += padding.as_str();
                buffer += "|\n";
                buffer += format!("{line} | {content}\n").as_str();
                buffer += padding.as_str();
                buffer += "| ";
            }
            None => {
                buffer += format!("{content}\n").as_str();
            }
        }
        for _ in 0..col - 1 {
            buffer += " ";
        }
        for _ in 0..self.length {
            buffer += "~";
        }
        write!(f, "{}", buffer,)
    }
}
