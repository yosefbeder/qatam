mod ast;
mod operators;
mod parser;
mod reporter;
mod token;
mod tokenizer;

use parser::Parser;
use reporter::{CliReporter, Reporter};
use std::fs;
use std::process::exit;
use tokenizer::Tokenizer;

fn run<'a>(source: &'a str, reporter: &mut dyn Reporter<'a>) {
    let mut tokenizer = Tokenizer::new(source);
    let mut parser = Parser::new(&mut tokenizer, reporter);
    match parser.parse() {
        Ok(ast) => {
            for stml in ast {
                print!("{}", stml);
            }
        }
        Err(_) => {}
    };
}

fn main() {
    let mut args = std::env::args().skip(1);
    let subcommand = args.next().unwrap_or_else(|| {
        eprintln!("توقعت أمراً الفرعية");
        exit(exitcode::USAGE);
    });
    match subcommand.as_str() {
        "نفذ" => {
            let path = args.next().unwrap_or_else(|| {
                eprintln!("توقعت مسار الملف");
                exit(exitcode::USAGE);
            });
            if args.next().is_some() {
                eprintln!("عدد غير متوقع من المدخلات");
                exit(exitcode::USAGE);
            }
            let source = fs::read_to_string(&path).unwrap_or_else(|err| {
                eprintln!("خطأ في قراءة الملف: {}", err);
                exit(exitcode::IOERR);
            });
            let mut cli_reporter = CliReporter::new();
            run(&source, &mut cli_reporter);
        }
        "ساعد" => {
            println!("{}", fs::read_to_string("help.txt").unwrap());
        }
        _ => {
            eprintln!("أمر فرعي غير متوقع");
            exit(exitcode::USAGE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parser::Parser,
        reporter::{Report, Reporter},
        tokenizer::Tokenizer,
    };

    pub struct ErrorsTracker<'a> {
        errors: Vec<Report<'a>>,
        warnings: Vec<Report<'a>>,
    }

    impl<'a> ErrorsTracker<'a> {
        pub fn new() -> Self {
            ErrorsTracker {
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        }
    }

    impl<'a> Reporter<'a> for ErrorsTracker<'a> {
        fn warning(&mut self, report: Report<'a>) {
            println!("تحذير: {}", report);
            self.warnings.push(report);
        }

        fn error(&mut self, report: Report<'a>) {
            eprintln!("خطأ {}: {}", report.phase, report);
            self.errors.push(report);
        }
    }

    #[test]
    fn parsing_exprs() {
        fn test_valid_expr(input: &'static str, expected: &'static str) {
            let mut errors_tracker = ErrorsTracker::new();
            let mut tokenizer = Tokenizer::new(input);
            let mut parser = Parser::new(&mut tokenizer, &mut errors_tracker);
            let expr = match parser.parse_expr() {
                Ok(expr) => expr,
                Err(_) => {
                    for report in errors_tracker.errors {
                        println!("{:?}", report);
                    }
                    panic!("Parsing {} failed", input);
                }
            };
            assert_eq!(expr.to_string(), expected);
        }

        fn test_invalid_expr(input: &'static str, expected_error: &'static str) {
            let mut errors_tracker = ErrorsTracker::new();
            let mut tokenizer = Tokenizer::new(input);
            let mut parser = Parser::new(&mut tokenizer, &mut errors_tracker);
            match parser.parse_expr() {
                Ok(_) => panic!("Parsing {} succeeded, but it should have failed", input),
                Err(_) => {
                    assert_eq!(errors_tracker.errors[0].msg, expected_error);
                }
            };
        }

        // precedence
        test_valid_expr("-أضف(3، 2).القيمة", "(- (أحضر (استدعي أضف [3 2]) القيمة))");
        test_valid_expr("1 + 2 * 3", "(+ 1 (* 2 3))");
        test_valid_expr("4 == 4 && صحيح || خطأ", "(|| (&& (== 4 4) صحيح) خطأ)");

        // associativity
        test_valid_expr("1 + 2 + 3", "(+ (+ 1 2) 3)");
        test_valid_expr("س = ص = ع", "(= س (= ص ع))");

        // parentheses
        test_valid_expr("(1 + 2) * 3", "(* (+ 1 2) 3)");

        // setters and '='
        test_valid_expr("س.س = 3", "(إجعل س س 3)");
        test_valid_expr("س = 3", "(= س 3)");
        test_invalid_expr("3 + س = 4", "الجانب الأيمن غير صحيح");
        test_invalid_expr("س + 3 = 4", "الجانب الأيمن غير صحيح");
        test_invalid_expr("3 + س.س = 4", "الجانب الأيمن غير صحيح");
        test_invalid_expr("س.س + 3 = 4", "الجانب الأيمن غير صحيح");

        // random errors
        test_invalid_expr("[3، 2", "توقعت ']' بعد القائمة");
        test_invalid_expr("{الاسم: \"يوسف\"", "توقعت '}' بعد القائمة");
        test_invalid_expr("{الاسم: \"يوسف\"،", "توقعت اسم الخاصية");
        test_invalid_expr("{الاسم: \"يوسف\" العمر: 16}", "توقعت '}' بعد القائمة"); //TODO improve this one
        test_invalid_expr("{الاسم: ", "توقعت عبارة");
        test_invalid_expr("{الاسم ", "توقعت ':' بعد الاسم");
        test_invalid_expr("{4 ", "توقعت اسم الخاصية");
    }

    #[test]
    fn parsing_stmls() {
        fn test_valid_stml(input: &'static str, msg: &'static str) {
            let mut errors_tracker = ErrorsTracker::new();
            let mut tokenizer = Tokenizer::new(input);
            let ast = Parser::new(&mut tokenizer, &mut errors_tracker).parse();
            assert!(ast.is_ok(), "{}", msg);
            println!("{:#?}", ast.unwrap());
        }

        test_valid_stml(
            "دالة أضف(الأول، الثاني) { أرجع الأول + الثاني ألقي \"هذا رائع\" }",
            "Parses functions",
        );
        test_valid_stml(
            "إن (س == 5) { إطبع(\"س تساوي 5\") } إلا { إطبع(\"س لا تساوي 5\") }",
            "Parses if-else statements",
        );
        test_valid_stml(
            "بينما (صحيح) { إطبع(\"إلا الأبد\") } كرر { إطبع(\"إطبع إلا الأبد\") أكمل قف }",
            "Parses loops",
        );
        test_valid_stml(
            "حاول { س = س / 0 } أمسك(الخطأ) { إطبع(الخطأ) }",
            "Parses try-catch",
        );
        test_valid_stml("{ إطبع(عدم) }", "Parses try-catch");
    }
}
