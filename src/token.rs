use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    NewLine,
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
    Fun,      // دالة
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

    Unkown,
    EOF,
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
