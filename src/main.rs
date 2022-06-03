mod ast;
mod chunk;
mod compiler;
mod natives;
mod operators;
mod parser;
mod reporter;
mod token;
mod tokenizer;
mod utils;
mod value;
mod vm;

use compiler::Compiler;
use parser::Parser;
use reporter::{CliReporter, Reporter};
use rustyline::{error::ReadlineError, Editor};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process,
};
use tokenizer::Tokenizer;
use value::Function;
use vm::Vm;

fn main() {
    let mut args = env::args().skip(1);
    let mut reporter = CliReporter::new();
    let mut vm = Vm::new();

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "--الإصدار" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
            }
            "--ساعد" => {
                println!("{}", include_str!("../help.md"));
            }
            _ => {
                if run_file(&arg, &mut vm, &mut reporter).is_err() {
                    process::exit(exitcode::DATAERR);
                }
            }
        }
    } else {
        run_repl(&mut vm, &mut reporter);
    }
}

fn run_repl(vm: &mut Vm, reporter: &mut dyn Reporter) {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("قتام \\ ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }

                rl.add_history_entry(&line);
                run_line(line, vm, reporter).ok();
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn run_line(line: String, vm: &mut Vm, reporter: &mut dyn Reporter) -> Result<(), ()> {
    let line = compile(line, None, reporter)?;
    vm.call_function(line).unwrap();
    vm.run(reporter)
}

fn run_file(arg: &str, vm: &mut Vm, reporter: &mut dyn Reporter) -> Result<(), ()> {
    let path = generate_path(arg)?;
    if let Some(dir) = path.parent() {
        vm.working_dir = dir.to_owned();
    }
    let source = fs::read_to_string(&path).unwrap();
    let script = compile(source, Some(&path), reporter)?;
    vm.call_function(script).unwrap();
    vm.run(reporter)
}

/// returns the absolute path of the file if it exists.
fn generate_path(arg: &str) -> Result<PathBuf, ()> {
    let path = PathBuf::from(arg);

    if !path.is_file() {
        eprintln!("الملف غير موجود");
        return Err(());
    }

    if path.extension() != Some(OsStr::new("قتام")) {
        eprintln!("يجب أن يكون إمتداد الملف \"قتام\"");
        return Err(());
    }

    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(path.canonicalize().unwrap())
    }
}

fn compile(
    source: String,
    path: Option<&Path>,
    reporter: &mut dyn Reporter,
) -> Result<Function, ()> {
    let mut tokenizer = Tokenizer::new(source, path);
    let mut parser = Parser::new(&mut tokenizer);
    let ast = parser.parse(reporter)?;
    if cfg!(feature = "debug-ast") {
        for stml in &ast {
            print!("{:?}", stml);
        }
    }
    let mut compiler = Compiler::new(&ast);
    let script = compiler.compile(reporter)?;
    if cfg!(feature = "debug-bytecode") {
        print!("{:?}", script);
    }
    Ok(script)
}
