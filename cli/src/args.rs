use std::{convert::Into, env, fmt, option, path::PathBuf};

#[derive(Debug, Clone)]
enum Option {
    Version,
    Help,
    Untrusted,
}

const VERSION: &str = "--الإصدار";
const HELP: &str = "--ساعد";
const UNTRUSTED: &str = "--غير-موثوق";

impl TryFrom<&str> for Option {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            VERSION => Ok(Self::Version),
            HELP => Ok(Self::Help),
            UNTRUSTED => Ok(Self::Untrusted),
            _ => Err(()),
        }
    }
}

impl Into<&str> for Option {
    fn into(self) -> &'static str {
        match self {
            Self::Version => VERSION,
            Self::Help => HELP,
            Self::Untrusted => UNTRUSTED,
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Option(Option),
    Path(PathBuf),
}

#[derive(Debug, Clone)]
pub enum LexError {
    UnknownOption(String),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption(string) => write!(f, "لا يوجد شئ من الإعدادات يسمى {string}"),
        }
    }
}

fn lex(iter: &mut env::Args) -> Result<Vec<Token>, LexError> {
    iter.next();
    let mut tokens = vec![];
    while let Some(string) = iter.next() {
        match string.as_str() {
            x if x.starts_with("--") => match Option::try_from(x) {
                Ok(option) => tokens.push(Token::Option(option)),
                Err(_) => {
                    LexError::UnknownOption(string);
                }
            },
            path => tokens.push(Token::Path(PathBuf::from(path))),
        }
    }
    Ok(tokens)
}

#[derive(Debug, Clone)]
struct Args {
    options: Vec<Option>,
    path: option::Option<PathBuf>,
}

impl Args {
    fn new(options: Vec<Option>, path: option::Option<PathBuf>) -> Self {
        Self { options, path }
    }
}

fn parse(tokens: Vec<Token>) -> Args {
    let mut iter = tokens.iter().peekable();
    let mut options = vec![];
    while let Some(Token::Option(option)) = iter.peek() {
        options.push(option.to_owned());
        iter.next();
    }
    let path = if let Some(Token::Path(path)) = iter.next() {
        Some(path.to_owned())
    } else {
        None
    };
    Args::new(options, path)
}

#[derive(Debug, Clone)]
pub enum EvalMode {
    File(PathBuf, bool),
    Repl,
}

#[derive(Clone)]
pub enum Action {
    Eval(EvalMode),
    Version,
    Help,
}

#[derive(Debug, Clone)]
pub enum CompileError {
    ExpectedPath,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedPath => write!(f, "توقعت مسار لملف"),
        }
    }
}

impl TryFrom<Args> for Action {
    type Error = CompileError;
    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let mut expect_path = false;
        let mut untrusted = false;
        for option in value.options {
            match option {
                Option::Help => return Ok(Self::Help),
                Option::Version => return Ok(Self::Version),
                Option::Untrusted => {
                    expect_path = true;
                    untrusted = true;
                }
            }
        }
        match value.path {
            Some(path) => Ok(Self::Eval(EvalMode::File(path, untrusted))),
            None => {
                if expect_path {
                    Err(CompileError::ExpectedPath)
                } else {
                    Ok(Self::Eval(EvalMode::Repl))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Lex(LexError),
    Compile(CompileError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(err) => write!(f, "{err}"),
            Self::Compile(err) => write!(f, "{err}"),
        }
    }
}

impl From<LexError> for Error {
    fn from(err: LexError) -> Self {
        Self::Lex(err)
    }
}

impl From<CompileError> for Error {
    fn from(err: CompileError) -> Self {
        Self::Compile(err)
    }
}

pub fn get_action() -> Result<Action, Error> {
    let tokens = lex(&mut env::args())?;
    let args = parse(tokens);
    Ok(Action::try_from(args)?)
}
