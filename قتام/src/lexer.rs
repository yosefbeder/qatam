use super::token::{Token, TokenType};
use std::{fmt, path::PathBuf, rc::Rc, result};

pub type Result = result::Result<Token, LexicalError>;

#[derive(Debug, Clone)]
pub enum LexicalError {
    Unknown(Token),
    UnTermedString(Token),
}

impl LexicalError {
    pub fn get_token(&self) -> &Token {
        match self {
            Self::Unknown(token) => token,
            Self::UnTermedString(token) => token,
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token = self.get_token();
        let typ: &str = token.typ.to_owned().into();
        write!(f, "{typ} {}", token.get_pos())
    }
}

pub struct Lexer {
    source: Rc<String>,
    path: Option<PathBuf>,
    start: usize,
    current: usize,
}

fn is_whitespace(c: char) -> bool {
    [
        '\u{0009}', '\u{000b}', '\u{000c}', '\u{0020}', '\u{0085}', '\u{200e}', '\u{200f}',
    ]
    .contains(&c)
}

fn is_newline(c: char) -> bool {
    ['\u{000a}', '\u{000d}', '\u{2028}', '\u{2029}'].contains(&c)
}

impl Lexer {
    pub fn new(source: String, path: Option<PathBuf>) -> Self {
        Self {
            source: Rc::new(source),
            path,
            start: 0,
            current: 0,
        }
    }

    fn peek(&self, distance: usize) -> Option<char> {
        self.source.chars().nth(self.current + distance)
    }

    fn at_end(&self) -> bool {
        self.peek(0).is_none()
    }

    fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current - 1)
    }

    fn check(&mut self, expected_c: char) -> bool {
        match self.peek(0) {
            Some(c) => c == expected_c,
            None => false,
        }
    }

    fn check_newline(&mut self) -> bool {
        match self.peek(0) {
            Some(c) => is_newline(c),
            None => false,
        }
    }

    fn pop_identifier(&mut self) -> Token {
        let start = self.start;
        let length = self.current - self.start;
        let lexeme = self.slice(start, length);
        self.start = self.current;
        Token::new(
            match lexeme.as_str() {
                "عدم" => TokenType::Nil,
                "صحيح" => TokenType::True,
                "خطأ" => TokenType::False,
                "إن" => TokenType::If,
                "وإن" => TokenType::ElseIf,
                "إلا" => TokenType::Else,
                "دالة" => TokenType::Function,
                "متغير" => TokenType::Var,
                "كرر" => TokenType::Loop,
                "طالما" => TokenType::While,
                "إكسر" => TokenType::Break,
                "واصل" => TokenType::Continue,
                "أرجع" => TokenType::Return,
                "ألقي" => TokenType::Throw,
                "حاول" => TokenType::Try,
                "أمسك" => TokenType::Catch,
                "استورد" => TokenType::Import,
                "من" => TokenType::From,
                "صدّر" => TokenType::Export,
                "لكل" => TokenType::For,
                "في" => TokenType::In,
                "و" => TokenType::And,
                "أو" => TokenType::Or,
                _ => TokenType::Identifier,
            },
            Rc::clone(&self.source),
            &self.path,
            lexeme,
            start,
        )
    }

    fn pop_token(&mut self, typ: TokenType) -> Result {
        let start = self.start;
        let length = self.current - self.start;
        let lexeme = self.slice(start, length);
        self.start = self.current;
        let token = Token::new(typ, Rc::clone(&self.source), &self.path, lexeme, start);
        match typ {
            TokenType::Unknown => Err(LexicalError::Unknown(token)),
            TokenType::UnTermedString => Err(LexicalError::UnTermedString(token)),
            _ => Ok(token),
        }
    }

    fn slice(&self, start: usize, length: usize) -> String {
        self.source
            .chars()
            .skip(start)
            .take(length)
            .collect::<String>()
    }

    fn pop_unknown_token(&mut self) -> Result {
        return self.pop_token(TokenType::Unknown);
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek(0) {
            match c {
                x if is_whitespace(x) => {
                    self.next();
                    self.start = self.current;
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result {
        self.skip_whitespace();

        if let Some(c) = self.next() {
            match c {
                x if is_newline(x) => self.pop_token(TokenType::NewLine),
                '(' => self.pop_token(TokenType::OParen),
                ')' => self.pop_token(TokenType::CParen),
                '{' => self.pop_token(TokenType::OBrace),
                '}' => self.pop_token(TokenType::CBrace),
                '[' => self.pop_token(TokenType::OBracket),
                ']' => self.pop_token(TokenType::CBracket),
                '.' => self.pop_token(TokenType::Period),
                '+' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::PlusEqual)
                    } else if self.check('+') {
                        self.next();
                        self.pop_token(TokenType::DPlus)
                    } else {
                        self.pop_token(TokenType::Plus)
                    }
                }
                '-' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::MinusEqual)
                    } else if self.check('-') {
                        self.next();
                        self.pop_token(TokenType::DMinus)
                    } else {
                        self.pop_token(TokenType::Minus)
                    }
                }
                '*' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::StarEqual)
                    } else {
                        self.pop_token(TokenType::Star)
                    }
                }
                '/' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::SlashEqual)
                    } else {
                        self.pop_token(TokenType::Slash)
                    }
                }
                '%' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::PercentEqual)
                    } else {
                        self.pop_token(TokenType::Percent)
                    }
                }
                '،' => self.pop_token(TokenType::Comma),
                '؟' => self.pop_token(TokenType::QuestionMark),
                ':' => self.pop_token(TokenType::Colon),
                '|' => self.pop_token(TokenType::Pipe),
                '=' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::DEqual)
                    } else {
                        self.pop_token(TokenType::Equal)
                    }
                }
                '!' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::BangEqual)
                    } else {
                        self.pop_token(TokenType::Bang)
                    }
                }
                '>' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::GreaterEqual)
                    } else {
                        self.pop_token(TokenType::Greater)
                    }
                }
                '<' => {
                    if self.check('=') {
                        self.next();
                        self.pop_token(TokenType::LessEqual)
                    } else {
                        self.pop_token(TokenType::Less)
                    }
                }
                '"' => {
                    while let Some(c) = self.next() {
                        match c {
                            '"' => {
                                return self.pop_token(TokenType::String);
                            }
                            x if is_newline(x) => {
                                let token = self.pop_token(TokenType::UnTermedString);
                                return token;
                            }
                            '\\' => match self.peek(0) {
                                Some('"') => {
                                    self.next();
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }

                    let token = self.pop_token(TokenType::UnTermedString);
                    return token;
                }
                '#' => {
                    while !self.at_end() && !self.check_newline() {
                        self.next();
                    }
                    self.pop_token(TokenType::Comment)
                }
                _ => {
                    if c.is_alphabetic() || c == '_' {
                        while let Some(c) = self.peek(0) {
                            if c.is_alphanumeric() || c == '_' {
                                self.next();
                            } else {
                                break;
                            }
                        }

                        return Ok(self.pop_identifier());
                    }

                    if c.is_ascii_digit() {
                        while let Some(c) = self.peek(0) {
                            if c.is_ascii_digit() {
                                self.next();
                            } else {
                                break;
                            }
                        }

                        if let Some('.') = self.peek(0) {
                            if let Some(c) = self.peek(1) {
                                if c.is_ascii_digit() {
                                    self.next();
                                    self.next();
                                    while let Some(c) = self.peek(0) {
                                        if c.is_ascii_digit() {
                                            self.next();
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        return self.pop_token(TokenType::Number);
                    }

                    self.pop_unknown_token()
                }
            }
        } else {
            self.pop_token(TokenType::EOF)
        }
    }
}
