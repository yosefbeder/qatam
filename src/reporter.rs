use super::token::Token;
use std::fmt;

#[derive(Debug)]
pub enum Phase {
    Tokenizing,
    Parsing,
    Runtime,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Tokenizing => write!(f, "مسحي"),
            Phase::Parsing => write!(f, "تحليلي"),
            Phase::Runtime => write!(f, "تنفيذي"),
        }
    }
}

#[derive(Debug)]
pub struct Report<'a> {
    pub phase: Phase,
    pub msg: String,
    pub token: Token<'a>,
}

impl<'a> Report<'a> {
    pub fn new(phase: Phase, msg: String, token: Token<'a>) -> Self {
        Report { phase, msg, token }
    }
}

impl<'a> fmt::Display for Report<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.token.get_pos();
        write!(f, "{} [{}:{}]\n{}", self.msg, line, col, self.token)
    }
}

pub trait Reporter<'a> {
    fn warning(&mut self, report: Report<'a>);
    fn error(&mut self, report: Report<'a>);
}
