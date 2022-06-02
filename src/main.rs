mod ast;
mod chunk;
mod compiler;
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

    if cfg!(feature = "debug-ast") {
        for stml in &ast {
            print!("{:?}", stml);
        }
    }

    let script = Compiler::new(&ast, reporter).compile().unwrap_or_else(|_| {
        eprintln!("توقفت جراء خطأ ترجمي");
        process::exit(exitcode::DATAERR);
    });

    if cfg!(feature = "debug-bytecode") {
        print!("{:?}", script);
    }

    Vm::new(script, reporter).run().unwrap_or_else(|_| {
        eprintln!("توقفت جراء خطأ تشغيلي");
        process::exit(exitcode::DATAERR);
    });

    println!("تمت العملية بنجاح 👍");
    process::exit(exitcode::OK);
}
