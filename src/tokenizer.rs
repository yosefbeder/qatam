use super::token::{Token, TokenType};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct Tokenizer {
    source: Rc<String>,
    path: Option<PathBuf>,
    start: usize,
    current: usize,
}

impl Tokenizer {
    pub fn new(source: String, path: Option<&Path>) -> Self {
        Self {
            source: Rc::new(source),
            path: match path {
                Some(path) => Some(path.to_owned()),
                None => None,
            },
            start: 0,
            current: 0,
        }
    }

    //TODO improve the performance of these helper functions
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
                "قف" => TokenType::Break,
                "أكمل" => TokenType::Continue,
                "أرجع" => TokenType::Return,
                "ألقي" => TokenType::Throw,
                "جرب" => TokenType::Try,
                "أمسك" => TokenType::Catch,
                _ => TokenType::Identifier,
            },
            Rc::clone(&self.source),
            &self.path,
            lexeme,
            start,
            length,
        )
    }

    fn pop_token(&mut self, typ: TokenType) -> Token {
        let start = self.start;
        let length = self.current - self.start;
        let lexeme = self.slice(start, length);
        self.start = self.current;
        Token::new(
            typ,
            Rc::clone(&self.source),
            &self.path,
            lexeme,
            start,
            length,
        )
    }

    fn slice(&self, start: usize, length: usize) -> String {
        self.source
            .chars()
            .skip(start)
            .take(length)
            .collect::<String>()
    }

    fn pop_unknown_token(&mut self) -> Token {
        return self.pop_token(TokenType::Unknown);
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek(0) {
            match c {
                '\r' | '\t' | ' ' => {
                    self.next();
                    self.start = self.current;
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(c) = self.next() {
            match c {
                '\n' => self.pop_token(TokenType::NewLine),
                '(' => self.pop_token(TokenType::OParen),
                ')' => self.pop_token(TokenType::CParen),
                '{' => self.pop_token(TokenType::OBrace),
                '}' => self.pop_token(TokenType::CBrace),
                '[' => self.pop_token(TokenType::OBracket),
                ']' => self.pop_token(TokenType::CBracket),
                '.' => self.pop_token(TokenType::Period),
                '+' => self.pop_token(TokenType::Plus),
                '-' => self.pop_token(TokenType::Minus),
                '*' => self.pop_token(TokenType::Star),
                '/' => self.pop_token(TokenType::Slash),
                '%' => self.pop_token(TokenType::Percent),
                '،' => self.pop_token(TokenType::Comma),
                '؟' => self.pop_token(TokenType::QuestionMark),
                ':' => self.pop_token(TokenType::Colon),
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
                '&' => {
                    if self.check('&') {
                        self.next();
                        self.pop_token(TokenType::And)
                    } else {
                        self.pop_unknown_token()
                    }
                }
                '|' => {
                    if self.check('|') {
                        self.next();
                        self.pop_token(TokenType::Or)
                    } else {
                        self.pop_unknown_token()
                    }
                }
                '"' => {
                    while let Some(c) = self.next() {
                        match c {
                            '"' => {
                                return self.pop_token(TokenType::String);
                            }
                            '\n' => {
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
                    while !self.at_end() && !self.check('\n') {
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

                        return self.pop_identifier();
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

                        if let Some(c) = self.peek(0) {
                            if c.is_alphabetic() || c == '_' {
                                self.next();
                                while let Some(c) = self.peek(0) {
                                    if c.is_alphanumeric() || c == '_' {
                                        self.next();
                                    } else {
                                        break;
                                    }
                                }

                                let token = self.pop_token(TokenType::InvalidNumber);
                                return token;
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
