use std::convert::{From, Into};
use std::{env, fmt, path::PathBuf};

#[derive(Debug, Clone)]
enum Setting {
    Version,
    Help,
    Untrusted,
    Unknown(String),
}

const VERSION: &str = "--الإصدار";
const HELP: &str = "--ساعد";
const UNTRUSTED: &str = "--غير-موثوق";

impl From<String> for Setting {
    fn from(value: String) -> Self {
        match value.as_str() {
            VERSION => Self::Version,
            HELP => Self::Help,
            UNTRUSTED => Self::Untrusted,
            string => Self::Unknown(string.to_owned()),
        }
    }
}

impl Into<String> for Setting {
    fn into(self) -> String {
        match self {
            Self::Version => VERSION.to_owned(),
            Self::Help => HELP.to_owned(),
            Self::Untrusted => UNTRUSTED.to_owned(),
            Self::Unknown(string) => string,
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Setting(Setting),
    Path(PathBuf),
}

fn lex(iter: &mut env::Args) -> Result<Vec<Token>, ParseError> {
    iter.next();
    let mut tokens = vec![];
    while let Some(string) = iter.next() {
        match string.as_str() {
            x if x.starts_with("--") => tokens.push(Token::Setting(Setting::from(string))),
            path => tokens.push(Token::Path(PathBuf::from(path))),
        }
    }
    Ok(tokens)
}

#[derive(Debug, Clone)]
struct Args {
    settings: Vec<Setting>,
    path: Option<PathBuf>,
}

impl Args {
    fn new(settings: Vec<Setting>, path: Option<PathBuf>) -> Self {
        Self { settings, path }
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectedPathOrSetting(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedPathOrSetting(string) => {
                write!(
                    f,
                    "توقعت مسار ملف أو أحد الإعدادات ولكن حصلت على \"{string}\""
                )
            }
        }
    }
}

fn parse(tokens: Vec<Token>) -> Result<Args, ParseError> {
    let mut iter = tokens.iter().peekable();
    let mut settings = vec![];
    while let Some(Token::Setting(setting)) = iter.peek() {
        match setting {
            Setting::Unknown(string) => {
                return Err(ParseError::ExpectedPathOrSetting(string.clone()))
            }
            _ => {}
        }
        settings.push(setting.to_owned());
        iter.next();
    }
    let path = if let Some(Token::Path(path)) = iter.next() {
        Some(path.to_owned())
    } else {
        None
    };
    Ok(Args::new(settings, path))
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
        for setting in value.settings {
            match setting {
                Setting::Help => return Ok(Self::Help),
                Setting::Version => return Ok(Self::Version),
                Setting::Untrusted => {
                    expect_path = true;
                    untrusted = true;
                }
                _ => unreachable!(),
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
    Parse(ParseError),
    Compile(CompileError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "{err}"),
            Self::Compile(err) => write!(f, "{err}"),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<CompileError> for Error {
    fn from(err: CompileError) -> Self {
        Self::Compile(err)
    }
}

pub fn get_action() -> Result<Action, Error> {
    let tokens = lex(&mut env::args())?;
    let args = parse(tokens)?;
    Ok(Action::try_from(args)?)
}
