mod args;
mod ast;
mod chunk;
mod compiler;
mod lexer;
mod natives;
mod operators;
mod parser;
mod token;
mod utils;
mod value;
mod vm;

use args::Mode;
use compiler::Compiler;
use parser::Parser;
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
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                run_line(&mut vm, line).ok();
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn run_line(vm: &mut Vm, line: String) -> Result<(), ()> {
    vm.run(compile(line, None)?)
}

fn run_file(path: PathBuf, untrusted: bool) -> Result<(), ()> {
    let mut vm = Vm::new(untrusted);
    let source = fs::read_to_string(&path).map_err(|err| eprintln!("{err}"))?;
    vm.run(compile(source, Some(path))?)
}

fn compile(source: String, path: Option<PathBuf>) -> Result<Function, ()> {
    let mut parser = Parser::new(source, path.clone());
    let ast = parser.parse()?;
    if cfg!(feature = "debug-ast") {
        for stml in &ast {
            println!("{:?}", stml);
        }
    }
    let mut compiler = Compiler::new(&ast, path);
    let script = compiler.compile()?;
    if cfg!(feature = "debug-bytecode") {
        print!("{:?}", script);
    }
    Ok(script)
}
