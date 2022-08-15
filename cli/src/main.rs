mod args;

use args::{get_action, Action, EvalMode};
use compiler::{Compiler, CompilerType};
use parser::Parser;
use rustyline::{error::ReadlineError, Editor};
use std::{fmt, fs, io, path::PathBuf};

const HELP_MSG: &str = "
طريقة الإستخدام:
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
    match try_main() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}")
        }
    }
}

fn try_main() -> Result<(), Error> {
    match get_action()? {
        Action::Eval(EvalMode::File(path, untrusted)) => file(path, untrusted)?,
        Action::Eval(EvalMode::Repl) => repl()?,
        Action::Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Action::Help => {
            println!(
                "{} {}\n{HELP_MSG}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            );
        }
    }
    Ok(())
}

enum Error {
    Args(args::Error),
    Parser(Vec<parser::Error>),
    Compiler(Vec<compiler::CompileError>),
    Readline(ReadlineError),
    Io(io::Error),
}

impl From<args::Error> for Error {
    fn from(err: args::Error) -> Self {
        Self::Args(err)
    }
}

impl From<Vec<parser::Error>> for Error {
    fn from(errors: Vec<parser::Error>) -> Self {
        Self::Parser(errors)
    }
}

impl From<ReadlineError> for Error {
    fn from(err: ReadlineError) -> Self {
        Self::Readline(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<Vec<compiler::CompileError>> for Error {
    fn from(errors: Vec<compiler::CompileError>) -> Self {
        Self::Compiler(errors)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Args(err) => write!(
                f,
                "{err}\nلمعرفة كيفية استخدام اللغة بطريقة صحيحة إستخدم \"--ساعد\""
            ),
            Self::Parser(errors) => {
                let mut iter = errors.iter();
                write!(f, "{}", iter.next().unwrap())?;
                while let Some(error) = iter.next() {
                    write!(f, "\n{error}")?;
                }
                Ok(())
            }
            Self::Compiler(errors) => {
                let mut iter = errors.iter();
                write!(f, "{}", iter.next().unwrap())?;
                while let Some(error) = iter.next() {
                    write!(f, "\n{error}")?;
                }
                Ok(())
            }
            Self::Readline(err) => {
                write!(f, "{err:?}")
            }
            Self::Io(err) => {
                write!(f, "{err:?}")
            }
        }
    }
}

fn repl() -> Result<(), ReadlineError> {
    let mut rl = Editor::<()>::new()?;
    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match run(line, None, false) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{err}")
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn file(path: PathBuf, untrusted: bool) -> Result<(), Error> {
    let source = fs::read_to_string(&path)?;
    run(source, Some(path), untrusted)
}

fn run(source: String, path: Option<PathBuf>, _untrusted: bool) -> Result<(), Error> {
    let (ast, token) = Parser::new(source, path).parse()?;
    let _ = Compiler::new(CompilerType::Script, &ast, token).compile()?;
    Ok(())
}