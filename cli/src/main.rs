mod args;

use args::{get_action, Action, EvalMode};
// use compiler::value::Function;
use compiler::Compiler;
use exitcode::USAGE;
use parser::Parser;
use rustyline::Editor;
use std::{fs, path::PathBuf, process::exit};

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
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                run_source(line, None)
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn run_file(path: PathBuf, _untrusted: bool) {
    run_source(fs::read_to_string(&path).unwrap(), Some(path))
}

fn run_source(source: String, path: Option<PathBuf>) {
    let (ast, token) = match Parser::new(source, path).parse() {
        Ok((ast, token)) => (ast, token),
        Err(errors) => {
            for err in errors {
                eprintln!("{err}")
            }
            return;
        }
    };
    match Compiler::new(compiler::CompilerType::Script, &ast, token).compile() {
        Ok(chunk) => print!("{chunk:?}"),
        Err(errors) => {
            for err in errors {
                eprintln!("{err}")
            }
        }
    }
}
