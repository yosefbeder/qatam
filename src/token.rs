use std::convert::From;
use std::fmt;

pub const NUMBER: usize = 48;

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
    EOF,
}

impl From<TokenType> for usize {
    fn from(typ: TokenType) -> usize {
        match typ {
            TokenType::OParen => 0,
            TokenType::CParen => 1,
            TokenType::OBrace => 2,
            TokenType::CBrace => 3,
            TokenType::OBracket => 4,
            TokenType::CBracket => 5,
            TokenType::Period => 6,
            TokenType::Plus => 7,
            TokenType::Minus => 8,
            TokenType::Star => 9,
            TokenType::Slash => 10,
            TokenType::Percent => 11,
            TokenType::Comma => 12,
            TokenType::QuestionMark => 13,
            TokenType::Colon => 14,

            TokenType::Equal => 15,
            TokenType::DEqual => 16,
            TokenType::Bang => 17,
            TokenType::BangEqual => 18,
            TokenType::Greater => 19,
            TokenType::GreaterEqual => 20,
            TokenType::Less => 21,
            TokenType::LessEqual => 22,
            TokenType::And => 23,
            TokenType::Or => 24,

            TokenType::String => 25,
            TokenType::UnTermedString => 26,
            TokenType::Comment => 27,

            TokenType::Identifier => 28,
            TokenType::If => 29,
            TokenType::Else => 30,
            TokenType::Function => 31,
            TokenType::Loop => 32,
            TokenType::While => 33,
            TokenType::Do => 34,
            TokenType::Break => 35,
            TokenType::Continue => 36,
            TokenType::Return => 37,
            TokenType::Throw => 38,
            TokenType::Try => 39,
            TokenType::Catch => 40,
            TokenType::Nil => 41,
            TokenType::True => 42,
            TokenType::False => 43,
            TokenType::Number => 44,
            TokenType::InvalidNumber => 45,

            TokenType::Unknown => 46,
            TokenType::EOF => 47,
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

pub const INVALID_TYPES: [TokenType; 3] = [
    TokenType::UnTermedString,
    TokenType::InvalidNumber,
    TokenType::Unknown,
];

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
