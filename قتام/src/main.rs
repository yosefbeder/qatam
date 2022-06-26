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

use compiler::Compiler;
use parser::Parser;
use path::{qatam_path, resolve_path};
use reporter::{CliReporter, Reporter};
use rustyline::Editor;
use std::{env, fs, path::PathBuf, process};
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
                if let Some(arg) = args.next() {
                    if arg.as_str().starts_with("--الدوال-المستبعدة=") {
                        let excluded = arg
                            .as_str()
                            .split("=")
                            .nth(1)
                            .unwrap()
                            .split("،")
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        if let Err(err) = vm.exclude_natives(excluded) {
                            eprintln!("{err}");
                            process::exit(exitcode::USAGE)
                        }
                    }
                }

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
            Err(_) => {
                break;
            }
        }
    }
}

fn run_line(line: String, vm: &mut Vm, reporter: &mut dyn Reporter) -> Result<(), ()> {
    vm.run(compile(line, None, reporter)?, reporter)
}

fn run_file(arg: &str, vm: &mut Vm, reporter: &mut dyn Reporter) -> Result<(), ()> {
    let path = resolve_path(None, arg, qatam_path).map_err(|err| eprintln!("{err}"))?;
    let source = fs::read_to_string(&path).unwrap();
    vm.run(compile(source, Some(path), reporter)?, reporter)
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
