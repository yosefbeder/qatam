use std::convert::From;
use std::fmt;

pub const NUMBER: usize = 49;

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
    If,       // إن
    Else,     // إلا
    Function, // دالة
    Loop,     // كرر
    While,    // بينما
    Do,       // إفعل
    Break,    // قف
    Continue, // أكمل
    Return,   // أرجع
    Throw,    // ألقي
    Try,      // حاول
    Catch,    // أمسك
    Nil,      // عدم
    True,     // صحيح
    False,    // خطأ
    Number,
    InvalidNumber,

    Unknown,
    NewLine,
    EOF,
}

pub const INVALID_TYPES: [TokenType; 3] = [
    TokenType::UnTermedString,
    TokenType::InvalidNumber,
    TokenType::Unknown,
];

pub const STATEMENT_BOUNDRIES: [TokenType; 10] = [
    TokenType::Function,
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
            TokenType::Else => 31,
            TokenType::Function => 32,
            TokenType::Loop => 33,
            TokenType::While => 34,
            TokenType::Do => 35,
            TokenType::Break => 36,
            TokenType::Continue => 37,
            TokenType::Return => 38,
            TokenType::Throw => 39,
            TokenType::Try => 40,
            TokenType::Catch => 41,
            TokenType::Nil => 42,
            TokenType::True => 43,
            TokenType::False => 44,
            TokenType::Number => 45,
            TokenType::InvalidNumber => 46,

            TokenType::Unknown => 47,
            TokenType::EOF => 48,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub typ: TokenType,
    source: &'a str,
    start: usize,
    length: usize,
}

impl<'a> Token<'a> {
    pub fn new(typ: TokenType, source: &'a str, start: usize, length: usize) -> Self {
        Self {
            typ,
            source,
            start,
            length,
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

    pub fn get_lexeme(&self) -> String {
        self.source
            .chars()
            .skip(self.start)
            .take(self.length)
            .collect::<String>()
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (line, col) = self.get_pos();

        write!(
            f,
            "{}\n{}^-هنا",
            match self.source.lines().nth(line - 1) {
                Some(line) => line.to_string(),
                None => "".to_string(),
            },
            " ".repeat(col - 1)
        )
    }
}
