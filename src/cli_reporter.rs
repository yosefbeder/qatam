use slia::reporter::{Report, Reporter};

pub fn to_arabic(digit: usize) -> String {
    let mut result = String::new();

    for c in digit.to_string().chars() {
        match c {
            '0' => result.push_str("٠"),
            '1' => result.push_str("١"),
            '2' => result.push_str("٢"),
            '3' => result.push_str("٣"),
            '4' => result.push_str("٤"),
            '5' => result.push_str("٥"),
            '6' => result.push_str("٦"),
            '7' => result.push_str("٧"),
            '8' => result.push_str("٨"),
            '9' => result.push_str("٩"),
            _ => unreachable!(),
        }
    }

    result
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
    fn report_warning(&mut self, report: Report) {
        self.warnings_count += 1;
        let (line, col) = report.token.get_pos();
        println!(
            "تحذير: {} [{}:{}]\n{}",
            report.msg,
            to_arabic(col),
            to_arabic(line),
            report.token
        );
    }

    fn report_error(&mut self, report: Report) {
        self.errors_count += 1;
        let (line, col) = report.token.get_pos();
        eprintln!(
            "خطأ {}: {} [{}:{}]\n{}",
            report.phase,
            report.msg,
            to_arabic(col),
            to_arabic(line),
            report.token
        );
    }
}
