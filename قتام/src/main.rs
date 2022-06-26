mod args;
mod ast;
mod chunk;
mod compiler;
mod natives;
mod operators;
mod parser;
mod path;
mod reporter;
mod token;
mod tokenizer;
mod utils;
mod value;
mod vm;

use args::Mode;
use compiler::Compiler;
use parser::Parser;
use reporter::{CliReporter, Reporter};
use rustyline::Editor;
use std::{env, fs, path::PathBuf, process::exit};
use value::Function;
use vm::Vm;

fn main() {
    use Mode::*;

    let mode = Mode::try_from(env::args())
        .map_err(|_| exit(exitcode::USAGE))
        .unwrap();

    match mode {
        Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Help => print!("{}", include_str!("../help.md")),
        Repl => run_repl(),
        File { path, untrusted } => run_file(path, untrusted)
            .map_err(|_| exit(exitcode::DATAERR))
            .unwrap(),
    };
}

fn run_repl() {
    let mut vm = Vm::new(false);
    let mut reporter = CliReporter::new();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("قتام \\ ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }

                rl.add_history_entry(&line);
                run_line(&mut vm, &mut reporter, line).ok();
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn run_line(vm: &mut Vm, reporter: &mut dyn Reporter, line: String) -> Result<(), ()> {
    vm.run(compile(line, None, reporter)?, reporter)
}

fn run_file(path: PathBuf, untrusted: bool) -> Result<(), ()> {
    let mut vm = Vm::new(untrusted);
    let mut reporter = CliReporter::new();
    let source = fs::read_to_string(&path).unwrap();
    vm.run(compile(source, Some(path), &mut reporter)?, &mut reporter)
}

fn compile(
    source: String,
    path: Option<PathBuf>,
    reporter: &mut dyn Reporter,
) -> Result<Function, ()> {
    let mut parser = Parser::new(source, path.clone());
    let ast = parser.parse(reporter)?;
    if cfg!(feature = "debug-ast") {
        for stml in &ast {
            print!("{:?}", stml);
        }
    }
    let mut compiler = Compiler::new(&ast, path);
    let script = compiler.compile(reporter)?;
    if cfg!(feature = "debug-bytecode") {
        print!("{:?}", script);
    }
    Ok(script)
}
