mod token;

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
    start: usize,
    current: usize,
    path: Option<PathBuf>,
}

impl Lexer {
    pub fn new(source: String, path: Option<&PathBuf>) -> Self {
        Self {
            source: Rc::new(source),
            start: 0,
            current: 0,
            path: path.cloned(),
        }
    }

    fn pop_token(&mut self, typ: TokenType, start: usize, end: usize) -> Rc<Token> {
        Rc::new(Token::new(
            typ,
            Rc::clone(&self.source),
            self.path.as_ref(),
            end,
            start - end + 1,
        ))
    }

    fn pop_token_from_state(&mut self, typ: TokenType) -> Rc<Token> {
        self.pop_token(typ, self.start, self.current)
    }

    fn next(&mut self, chars: &mut Peekable<CharIndices>) -> Option<char> {
        let (offset, c) = chars.next()?;
        self.current = offset;
        Some(c)
    }

    fn skip_whitespace(&mut self, chars: &mut Peekable<CharIndices>) {
        while let Some((_, c)) = chars.peek().cloned() {
            if is_whitespace(c) {
                self.next(chars);
            } else {
                break;
            }
        }
    }

    // fn next_token(&mut self, chars: &mut Peekable<CharIndices>) -> Option<Rc<Token>> {
    //     self.skip_whitespace(chars);
    //     let mut c = self.next(chars)?;
    //     self.start = self.current;
    //     Some(match c {
    //         x if is_newline(x) => self.pop_token(TokenType::NewLine),
    //         '(' => self.pop_token(TokenType::OParen),
    //         ')' => self.pop_token(TokenType::CParen),
    //         '{' => self.pop_token(TokenType::OBrace),
    //         '}' => self.pop_token(TokenType::CBrace),
    //         '[' => self.pop_token(TokenType::OBracket),
    //         ']' => self.pop_token(TokenType::CBracket),
    //         '.' => {
    //             if let Some('.') = self.next(chars) {
    //                 if let Some('.') = self.next(chars) {

    //                 }
    //             } else {
    //                 self.pop_token(TokenType::)
    //             }
    //         }
    //         _ => self.pop_unknown(),
    //     })
    // }

    pub fn lex(mut self) -> Vec<Rc<Token>> {
        let source = Rc::clone(&self.source);
        let mut chars = source.char_indices().peekable();
        let mut tokens = vec![];
        while let Some(c) = self.next(&mut chars) {
            self.start = self.current;
            match c {
                x if is_newline(x) => tokens.push(self.pop_token(TokenType::NewLine)),
                '(' => tokens.push(self.pop_token(TokenType::OParen)),
                ')' => tokens.push(self.pop_token(TokenType::CParen)),
                '{' => tokens.push(self.pop_token(TokenType::OBrace)),
                '}' => tokens.push(self.pop_token(TokenType::CBrace)),
                '[' => tokens.push(self.pop_token(TokenType::OBracket)),
                ']' => tokens.push(self.pop_token(TokenType::CBracket)),
                '.' => {
                    if let Some('.') = self.next(chars) {
                        if let Some('.') = self.next(chars) {}
                    } else {
                        self.push()
                    }
                }
                _ => tokens.push(self.pop_token_from_state(TokenType::Unknown)),
            }
        }
        tokens
    }
}
