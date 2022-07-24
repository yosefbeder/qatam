mod args;

use args::{get_action, Action, EvalMode};
use compiler::value::Function;
use compiler::Compiler;
use exitcode::{DATAERR, USAGE};
use parser::Parser;
use rustyline::Editor;
use std::{fs, path::PathBuf, process::exit};
use vm::Vm;

const HELP_MSG: &str = "طريقة الإستخدام:
  قتام [الإعدادات] [الملف [مدخلات البرنامج]]

في حالة عدم توافر الملف ستعمل اللغة على الوضع التفاعلي.

الإعدادات:
  --غير-موثوق
    يمنع المستخدم من استخدام الخواص الخطيرة مثل قراءة الملفات وتغيير محتواها (لاحظ: يجب عليكم توفير الملف).
  --الإصدار
    يقوم بطباعة الإصدار المستخدم حالياً (لاحظ: هذا الأمر يتجاهل الملف).
  --ساعد
    يقوم بطباعة هذه الرسالة (لاحظ: هذا الأمر يتجاهل الملف).
";

fn main() {
    match get_action() {
        Ok(action) => match action {
            Action::Eval(EvalMode::File(path, untrusted)) => run_file(path, untrusted),
            Action::Eval(EvalMode::Repl) => run_repl(),
            Action::Version => println!("{}", env!("CARGO_PKG_VERSION")),
            Action::Help => {
                println!(
                    "{} {}\n\n{HELP_MSG}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                );
            }
        },
        Err(err) => {
            eprintln!("{err}");
            eprintln!("لمعرفة كيفية استخدام اللغة بطريقة صحيحة إستخدم '--ساعد'");
            exit(USAGE)
        }
    }
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

fn run_file(path: PathBuf, untrusted: bool) {
    let mut vm = Vm::new(untrusted);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("{err}");
            exit(DATAERR)
        }
    };
    let function = match compile(source, Some(path)) {
        Ok(function) => function,
        Err(_) => exit(DATAERR),
    };
    match vm.run(function) {
        Ok(_) => {}
        Err(_) => exit(DATAERR),
    }
}

fn compile(source: String, path: Option<PathBuf>) -> Result<Function, ()> {
    let mut parser = Parser::new(source, path.clone());
    let ast = parser.parse().map_err(|errors| {
        for err in errors {
            eprintln!("{err}");
        }
    })?;
    let mut compiler = Compiler::new(&ast, path);
    let script = compiler.compile().map_err(|errors| {
        for err in errors {
            eprintln!("{err}");
        }
    })?;
    Ok(script)
}
