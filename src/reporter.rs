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
pub struct Report {
    pub phase: Phase,
    pub msg: String,
    pub token: Rc<Token>,
}

impl Report {
    pub fn new(phase: Phase, msg: String, token: Rc<Token>) -> Self {
        Report { phase, msg, token }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.msg, self.token)
    }
}

pub trait Reporter {
    fn warning(&mut self, report: Report);
    fn error(&mut self, report: Report);
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

impl Reporter for CliReporter {
    fn warning(&mut self, report: Report) {
        self.warnings_count += 1;
        println!("تحذير: {}", report);
    }

    fn error(&mut self, report: Report) {
        self.errors_count += 1;
        eprintln!("خطأ {}: {}", report.phase, report);
    }
}
