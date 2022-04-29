use qatam::reporter::{Report, Reporter};

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
        let (line, col) = report.token.get_pos();
        println!("تحذير: {} [{}:{}]\n{}", report.msg, line, col, report.token);
    }

    fn error(&mut self, report: Report) {
        self.errors_count += 1;
        let (line, col) = report.token.get_pos();
        eprintln!(
            "خطأ {}: {} [{}:{}]\n{}",
            report.phase, report.msg, line, col, report.token
        );
    }
}
