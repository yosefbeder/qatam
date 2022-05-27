mod ast;
mod chunk;
mod compiler;
mod debug;
mod operators;
mod parser;
mod qatam;
mod reporter;
mod token;
mod tokenizer;
mod utils;
mod value;
mod vm;

use std::{env, fs, process};

fn main() {
    use reporter::CliReporter;

    let mut args = env::args().skip(1);
    let subcommand = args.next().unwrap_or_else(|| {
        eprintln!("توقعت أمراً الفرعية");
        process::exit(exitcode::USAGE);
    });
    match subcommand.as_str() {
        "نفذ" => {
            let path = args.next().unwrap_or_else(|| {
                eprintln!("توقعت مسار الملف");
                process::exit(exitcode::USAGE);
            });
            if args.next().is_some() {
                eprintln!("عدد غير متوقع من المدخلات");
                process::exit(exitcode::USAGE);
            }
            let source = fs::read_to_string(&path).unwrap_or_else(|err| {
                eprintln!("خطأ في قراءة الملف: {}", err);
                process::exit(exitcode::IOERR);
            });
            let mut cli_reporter = CliReporter::new();
            run_file(&source, &path, &mut cli_reporter);
        }
        "ساعد" => {
            println!("{}", fs::read_to_string("help.txt").unwrap());
        }
        _ => {
            eprintln!("أمر فرعي غير متوقع");
            process::exit(exitcode::USAGE);
        }
    }
}

pub fn run_file<'a>(source: &'a str, file: &str, reporter: &mut dyn reporter::Reporter<'a>) {
    use compiler::Compiler;
    use debug::{debug_ast, debug_bytecode};
    use parser::Parser;
    use tokenizer::Tokenizer;
    use vm::Vm;

    let mut tokenizer = Tokenizer::new(source, file);

    let ast = Parser::new(&mut tokenizer, reporter)
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("توقفت جراء خطأ تحليلي");
            process::exit(exitcode::DATAERR);
        });
    debug_ast(&ast);

    let script = Compiler::new(&ast, reporter).compile().unwrap_or_else(|_| {
        eprintln!("توقفت جراء خطأ ترجمي");
        process::exit(exitcode::DATAERR);
    });
    debug_bytecode(&script);

    Vm::new(script, reporter).run().unwrap_or_else(|_| {
        eprintln!("توقفت جراء خطأ تشغيلي");
        process::exit(exitcode::DATAERR);
    });

    println!("تمت العملية بنجاح 👍");
    process::exit(exitcode::OK);
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
            let mut tokenizer = Tokenizer::new(input, "test");
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
            assert_eq!(format!("{:?}", expr), expected);
        }

        fn test_invalid_expr(input: &'static str, expected_error: &'static str) {
            let mut errors_tracker = ErrorsTracker::new();
            let mut tokenizer = Tokenizer::new(input, "test");
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
}
