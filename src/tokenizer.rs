use super::reporter::{Phase, Report, Reporter};
use super::token::{Token, TokenType};
use std::rc::Rc;

pub struct Tokenizer<'a> {
    source: &'a str,
    start: usize,
    current: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
        }
    }

    fn error_at(token: &Token<'a>, msg: &str, reporter: &mut dyn Reporter<'a>) {
        let report = Report::new(Phase::Tokenizing, msg.to_string(), Rc::new(token.clone()));
        reporter.error(report);
    }

    fn warning_at(token: &Token<'a>, msg: &str, reporter: &mut dyn Reporter<'a>) {
        let report = Report::new(Phase::Tokenizing, msg.to_string(), Rc::new(token.clone()));
        reporter.warning(report);
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

    fn pop_token(&mut self, typ: TokenType) -> Token<'a> {
        let start = self.start;
        let length = self.current - self.start;
        self.start = self.current;
        Token::new(typ, self.source, start, length)
    }

    fn slice(&self, start: usize, length: usize) -> String {
        self.source
            .chars()
            .skip(start)
            .take(length)
            .collect::<String>()
    }

    fn pop_unknown_token(&mut self, reporter: &mut dyn Reporter<'a>) -> Token<'a> {
        let token = self.pop_token(TokenType::Unknown);
        Self::error_at(&token, "رمز غير متوقع", reporter);
        return token;
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

    pub fn next_token(&mut self, reporter: &mut dyn Reporter<'a>) -> Token<'a> {
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
                        self.pop_unknown_token(reporter)
                    }
                }
                '|' => {
                    if self.check('|') {
                        self.next();
                        self.pop_token(TokenType::Or)
                    } else {
                        self.pop_unknown_token(reporter)
                    }
                }
                '"' => {
                    let mut slash_misused = false;

                    while let Some(c) = self.next() {
                        match c {
                            '"' => {
                                if slash_misused {
                                    let token = self.pop_token(TokenType::String);
                                    Self::warning_at(&token, "خطأ في إستخدام '\\'", reporter);
                                    return token;
                                } else {
                                    return self.pop_token(TokenType::String);
                                }
                            }
                            '\n' => {
                                let token = self.pop_token(TokenType::UnTermedString);
                                Self::error_at(&token, "نص غير مغلق", reporter);
                                return token;
                            }
                            '\\' => match self.peek(0) {
                                Some('"') => {
                                    self.next();
                                }
                                Some('n') | Some('t') | Some('b') | Some('r') | Some('f')
                                | Some('\\') => {}
                                _ => {
                                    slash_misused = true;
                                }
                            },
                            _ => {}
                        }
                    }

                    let token = self.pop_token(TokenType::UnTermedString);
                    Self::error_at(&token, "نص غير مغلق", reporter);
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

                        return self.pop_token(
                            match &self.slice(self.start, self.current - self.start) as &str {
                                "عدم" => TokenType::Nil,
                                "صحيح" => TokenType::True,
                                "خطأ" => TokenType::False,
                                "إن" => TokenType::If,
                                "إلا" => TokenType::Else,
                                "دالة" => TokenType::Function,
                                "متغير" => TokenType::Var,
                                "كرر" => TokenType::Loop,
                                "بينما" => TokenType::While,
                                "إفعل" => TokenType::Do,
                                "قف" => TokenType::Break,
                                "أكمل" => TokenType::Continue,
                                "أرجع" => TokenType::Return,
                                "ألقي" => TokenType::Throw,
                                "حاول" => TokenType::Try,
                                "أمسك" => TokenType::Catch,
                                _ => TokenType::Identifier,
                            },
                        );
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
                                Self::error_at(&token, "رقم خاطئ", reporter);
                                return token;
                            }
                        }

                        return self.pop_token(TokenType::Number);
                    }

                    self.pop_unknown_token(reporter)
                }
            }
        } else {
            self.pop_token(TokenType::EOF)
        }
    }
}
