use super::token::Token;
use std::{fmt, rc::Rc};

#[derive(Debug)]
pub enum Phase {
    Tokenizing,
    Parsing,
    Compilation,
    Runtime,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Tokenizing => write!(f, "مسحي"),
            Phase::Parsing => write!(f, "تحليلي"),
            Phase::Compilation => write!(f, "ترجمي"),
            Phase::Runtime => write!(f, "تنفيذي"),
        }
    }
}

//TODO consider storing 'Token' in a reference counter
#[derive(Debug)]
pub struct Report<'a> {
    pub phase: Phase,
    pub msg: String,
    pub token: Rc<Token<'a>>,
}

impl<'a> Report<'a> {
    pub fn new(phase: Phase, msg: String, token: Rc<Token<'a>>) -> Self {
        Report { phase, msg, token }
    }
}

impl<'a> fmt::Display for Report<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.msg, self.token)
    }
}

pub trait Reporter<'a> {
    fn warning(&mut self, report: Report<'a>);
    fn error(&mut self, report: Report<'a>);
}

pub struct CliReporter {
    errors_count: usize,
    warnings_count: usize,
}

impl CliReporter {
    pub fn new() -> Self {
        Self {
            errors_count: 0,
            warnings_count: 0,
        }
    }
}

impl<'a> Reporter<'a> for CliReporter {
    fn warning(&mut self, report: Report) {
        self.warnings_count += 1;
        println!("تحذير: {}", report);
    }

    fn error(&mut self, report: Report) {
        self.errors_count += 1;
        eprintln!("خطأ {}: {}", report.phase, report);
    }
}
