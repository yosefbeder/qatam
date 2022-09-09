pub mod token;

use std::{iter::Peekable, path::PathBuf, rc::Rc, str::CharIndices};
use token::*;

fn is_whitespace(c: char) -> bool {
    [
        '\u{0009}', '\u{000b}', '\u{000c}', '\u{0020}', '\u{0085}', '\u{200e}', '\u{200f}',
    ]
    .contains(&c)
}

fn is_newline(c: char) -> bool {
    ['\u{000a}', '\u{000d}', '\u{2028}', '\u{2029}'].contains(&c)
}

pub struct Lexer {
    source: Rc<String>,
    path: Option<PathBuf>,
}

impl Lexer {
    pub fn new(source: String, path: Option<&PathBuf>) -> Self {
        Self {
            source: Rc::new(source),
            path: path.cloned(),
        }
    }

    /// Creates a new token returning it.
    ///
    /// `first` represents the offset of the first character, while `last` represents the offset of the last.
    fn pop_token(&mut self, typ: TokenType, first: usize, length: usize) -> Rc<Token> {
        Rc::new(Token::new(
            typ,
            Rc::clone(&self.source),
            self.path.as_ref(),
            first,
            length,
        ))
    }

    /// Returns the next character along with its offset without advancing the iterator.
    fn peek(char_indices: &mut Peekable<CharIndices>) -> Option<(usize, char)> {
        char_indices.peek().cloned()
    }

    /// Checks whether the next character matches `pred` or not without advancing the iterator.
    fn check(char_indices: &mut Peekable<CharIndices>, pred: Box<dyn Fn(char) -> bool>) -> bool {
        if let Some((_, c)) = Self::peek(char_indices) {
            pred(c)
        } else {
            false
        }
    }

    /// Returns the next character along with its offset advancing the iterator.
    fn next(char_indices: &mut Peekable<CharIndices>) -> Option<(usize, char)> {
        char_indices.next()
    }

    fn at_end(char_indices: &mut Peekable<CharIndices>) -> bool {
        Self::next(char_indices).is_none()
    }

    /// If the next character matches `pred`, Advances the iterator returning the next element.
    fn check_next(
        char_indices: &mut Peekable<CharIndices>,
        pred: Box<dyn Fn(char) -> bool>,
    ) -> Option<(usize, char)> {
        if Self::check(char_indices, pred) {
            Self::next(char_indices)
        } else {
            None
        }
    }

    /// Returns a predicator that matches a single character.
    fn is(expected: char) -> Box<dyn Fn(char) -> bool> {
        Box::new(move |c| c == expected)
    }

    /// Advances the iterator ignoring whitespaces.
    fn next_no_whitespace(char_indices: &mut Peekable<CharIndices>) -> Option<(usize, char)> {
        loop {
            if Self::check_next(char_indices, Box::new(is_whitespace)).is_none() {
                return Self::next(char_indices);
            }
        }
    }

    pub fn lex(mut self) -> Vec<Rc<Token>> {
        use TokenType::*;

        let source = Rc::clone(&self.source);
        let mut char_indices = source.char_indices().peekable();
        let mut tokens = vec![];
        while let Some((first, c)) = Self::next_no_whitespace(&mut char_indices) {
            macro_rules! single {
                ($typ:ident) => {
                    tokens.push(self.pop_token($typ, first, 1))
                };
            }
            macro_rules! optional_equal {
                ($without:ident, $with:ident) => {
                    if Self::check_next(&mut char_indices, Self::is('=')).is_some() {
                        tokens.push(self.pop_token($with, first, 2))
                    } else {
                        tokens.push(self.pop_token($without, first, 1))
                    }
                };
            }

            match c {
                x if is_newline(x) => single!(NewLine),
                '(' => single!(OParen),
                ')' => single!(CParen),
                '{' => single!(OBrace),
                '}' => single!(CBrace),
                '[' => single!(OBracket),
                ']' => single!(CBracket),
                '،' => single!(Comma),
                '؟' => single!(QuestionMark),
                ':' => single!(Colon),
                '|' => single!(Pipe),
                '+' => optional_equal!(Plus, PlusEqual),
                '-' => optional_equal!(Minus, MinusEqual),
                '*' => optional_equal!(Star, StarEqual),
                '/' => {
                    if Self::check_next(&mut char_indices, Self::is('=')).is_some() {
                        tokens.push(self.pop_token(SlashEqual, first, 2))
                    } else if Self::check_next(&mut char_indices, Self::is('/')).is_some() {
                        loop {
                            if let Some((last, _)) =
                                Self::check_next(&mut char_indices, Box::new(is_newline))
                            {
                                tokens.push(self.pop_token(InlineComment, first, first - last + 1));
                                break;
                            } else if Self::at_end(&mut char_indices) {
                                // TODO test
                                tokens.push(self.pop_token(
                                    InlineComment,
                                    first,
                                    self.source.len() - first,
                                ));
                                break;
                            } else {
                                Self::next(&mut char_indices);
                            }
                        }
                    } else if let Some((_, _)) = Self::check_next(&mut char_indices, Self::is('*'))
                    {
                        loop {
                            if Self::check_next(&mut char_indices, Self::is('*')).is_some() {
                                if let Some((last, _)) =
                                    Self::check_next(&mut char_indices, Self::is('/'))
                                {
                                    tokens.push(self.pop_token(
                                        BlockComment,
                                        first,
                                        first - last + 1,
                                    ));
                                    break;
                                }
                            } else if Self::at_end(&mut char_indices) {
                                tokens.push(self.pop_token(
                                    UnterminatedBlockComment,
                                    first,
                                    source.len() - first,
                                ));
                                break;
                            } else {
                                Self::next(&mut char_indices);
                            }
                        }
                    } else {
                        tokens.push(self.pop_token(Slash, first, 1))
                    }
                }
                '%' => optional_equal!(Percent, PercentEqual),
                '!' => optional_equal!(Bang, BangEqual),
                '=' => optional_equal!(Equal, DEqual),
                '>' => optional_equal!(Greater, GreaterEqual),
                '<' => optional_equal!(Less, LessEqual),
                '.' => {
                    if let Some((second_first, _)) =
                        Self::check_next(&mut char_indices, Self::is('.'))
                    {
                        if Self::check_next(&mut char_indices, Self::is('.')).is_some() {
                            tokens.push(self.pop_token(TPeriod, first, 3))
                        } else {
                            tokens.push(self.pop_token(Period, first, 1));
                            tokens.push(self.pop_token(Period, second_first, 1))
                        }
                    } else {
                        tokens.push(self.pop_token(Period, first, 1))
                    }
                }
                '"' => loop {
                    if let Some((last, _)) = Self::check_next(&mut char_indices, Self::is('"')) {
                        tokens.push(self.pop_token(String, first, first - last + 1));
                        break;
                    } else if Self::check_next(&mut char_indices, Self::is('\\')).is_some() {
                        Self::check_next(&mut char_indices, Box::new(|c| c == '"'));
                    } else if let Some((last, _)) =
                        Self::check_next(&mut char_indices, Box::new(is_newline))
                    {
                        tokens.push(self.pop_token(UnterminatedString, first, first - last + 1));
                        break;
                    } else if Self::at_end(&mut char_indices) {
                        tokens.push(self.pop_token(
                            UnterminatedString,
                            first,
                            source.len() - first,
                        ));
                        break;
                    } else {
                        Self::next(&mut char_indices);
                    }
                },
                x if x.is_alphabetic() || x == '_' => {
                    let mut last = first;
                    while let Some((offset, _)) = Self::check_next(
                        &mut char_indices,
                        Box::new(|c| c.is_alphanumeric() || c == '_'),
                    ) {
                        last = offset;
                    }
                    tokens.push(self.pop_token(Identifier, first, last))
                }
                x if x.is_ascii_digit() => {
                    let mut int_last = first;
                    while let Some((offset, _)) =
                        Self::check_next(&mut char_indices, Box::new(|c| c.is_ascii_digit()))
                    {
                        int_last = offset;
                    }
                    if let Some((offset, _)) = Self::check_next(&mut char_indices, Self::is('.')) {
                        if Self::check_next(&mut char_indices, Box::new(|c| c.is_ascii_digit()))
                            .is_some()
                        {
                            todo!("Floats")
                        } else {
                            tokens.push(self.pop_token(Number, first, int_last));
                            tokens.push(self.pop_token(Period, offset, offset));
                        }
                    } else {
                        tokens.push(self.pop_token(Number, first, int_last));
                    }
                }
                _ => single!(Unknown),
            }
        }
        tokens.push(Rc::new(Token::new(
            EOF,
            Rc::clone(&source),
            self.path.as_ref(),
            source.len() - 1,
            0,
        )));
        tokens
    }
}
