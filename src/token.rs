use std::cmp::PartialEq;
use std::convert::From;
use std::fmt;

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
    Const,
    Loop,
    While,
    Do,
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

    Unknown,
    NewLine,
    EOF,
}

pub const BOUNDARIES: [TokenType; 11] = [
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
            TokenType::Const => 35,
            TokenType::Loop => 36,
            TokenType::While => 37,
            TokenType::Do => 38,
            TokenType::Break => 39,
            TokenType::Continue => 40,
            TokenType::Return => 41,
            TokenType::Throw => 42,
            TokenType::Try => 43,
            TokenType::Catch => 44,
            TokenType::Nil => 45,
            TokenType::True => 46,
            TokenType::False => 47,
            TokenType::Number => 48,
            TokenType::InvalidNumber => 49,

            TokenType::Unknown => 50,
            TokenType::EOF => 51,
        }
    }
}

pub const NUMBER: usize = 52;

#[derive(Clone)]
pub struct Token<'a> {
    pub typ: TokenType,
    source: &'a str,
    file: String,
    pub lexeme: String,
    start: usize,
    length: usize,
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.get_pos();
        write!(
            f,
            "{{\n{3}type: {0:?},\n{3}lexeme: {1},\n{3}file:{2},\n{3}line: {line},\n{3}column: {col},\n}}",
            self.typ,
            self.lexeme,
            self.file,
            " ".repeat(4),
        )
    }
}

impl<'a> Token<'a> {
    pub fn new(
        typ: TokenType,
        source: &'a str,
        file: &str,
        lexeme: String,
        start: usize,
        length: usize,
    ) -> Self {
        Self {
            typ,
            source,
            file: file.to_string(),
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

impl<'a> PartialEq for Token<'a> {
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

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.get_pos();
        let padding = " ".repeat(format!("{line}").len() + 1);

        let mut buffer = String::new();
        buffer += format!("{}:{}:{}\n", self.file, line, col).as_str();
        buffer += padding.as_str();
        buffer += "|\n";
        buffer += format!("{line} | {}\n", self.source.lines().nth(line - 1).unwrap()).as_str();
        buffer += padding.as_str();
        buffer += "| ";
        for _ in 0..col - 1 {
            buffer += " ";
        }
        for _ in 0..self.length {
            buffer += "~";
        }
        write!(f, "{}", buffer,)
    }
}
