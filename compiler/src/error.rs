use super::value::{Arity, DataType, Value};
use colored::Colorize;
use parser::token::*;
use std::{fmt, io, rc::Rc};

#[derive(Debug, Clone)]
pub enum CompileError {
    TooManyConsts(Rc<Token>),
    HugeSize(Rc<Token>),
    BackSlashMisuse(Rc<Token>),
    DefaultInObject(Rc<Token>),
    HugeJump(Rc<Token>),
    TooManyLocals(Rc<Token>),
    TooManyUpvalues(Rc<Token>),
    SameVarInScope(Rc<Token>),
    InvalidDes(Rc<Token>),
    ReturnOutsideFunction(Rc<Token>),
    TooManyExports(Rc<Token>),
    OutsideLoopBreak(Rc<Token>),
    OutsideLoopContinue(Rc<Token>),
    InvalidImportUsage(Rc<Token>),
    InvalidExportUsage(Rc<Token>),
    Io(Rc<Token>, Rc<io::Error>),
    ModuleParser(Rc<Token>, Vec<parser::Error>),
    TooManyArgs(Rc<Token>),
}

impl TokenInside for CompileError {
    fn token(&self) -> Rc<Token> {
        match self {
            Self::TooManyConsts(token, ..)
            | Self::HugeSize(token, ..)
            | Self::BackSlashMisuse(token, ..)
            | Self::DefaultInObject(token, ..)
            | Self::HugeJump(token, ..)
            | Self::TooManyLocals(token, ..)
            | Self::TooManyUpvalues(token, ..)
            | Self::SameVarInScope(token, ..)
            | Self::InvalidDes(token, ..)
            | Self::ReturnOutsideFunction(token, ..)
            | Self::TooManyExports(token, ..)
            | Self::OutsideLoopBreak(token, ..)
            | Self::OutsideLoopContinue(token, ..)
            | Self::InvalidImportUsage(token, ..)
            | Self::InvalidExportUsage(token, ..)
            | Self::Io(token, ..)
            | Self::ModuleParser(token, ..)
            | Self::TooManyArgs(token, ..) => Rc::clone(token),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "خطأ ترجمي: ".bright_red())?;
        match self {
            Self::TooManyConsts(token) => {
                writeln!(f, "لا يمكن أن تحتوي الدالة الواحدة على أكثر من 65536  ثابت")?;
                write!(f, "{token}")
            }
            Self::HugeSize(token) => {
                write!(f, "لا يمكن أن  ")?;
                match token.typ() {
                    TokenType::OBracket => write!(f, "تنشأ قائمة جديدة ")?,
                    TokenType::OBrace => write!(f, "ينشأ كائن جديد ")?,
                    _ => unreachable!(),
                }
                writeln!(f, "بأكثر من 65535 عنصر")?;
                write!(f, "{token}")
            }
            Self::BackSlashMisuse(token) => {
                writeln!(f, "استعمال خاطئ ل\"\\\"")?;
                writeln!(f, "{token}")?;
                write!(
                    f,
                    "حيث يمكن أن تكون متلية فقط ب\"n\" أو \"r\" أو \"t\" أو '\"'"
                )
            }
            Self::DefaultInObject(token) => {
                writeln!(
                    f,
                    "لا يمكن أن يحتوي كائن على قيمة إفتراضية - حيث أنها تكون فقط في التوزيع -"
                )?;
                write!(f, "{token}")
            }
            Self::HugeJump(token) => {
                writeln!(f, "لا يمكن القفز فوق أكثر من 65533 بايت")?;
                writeln!(f, "{token}")?;
                write!(f, "إقتراح: إن حدث هذا الخطأ في شرط أو تكرار يمكنك تصغير حجم جسمه بإنشاء بعض الدوال")
            }
            Self::TooManyLocals(token) => {
                writeln!(f, "لا يمكن أن تحتوي دالة على أكثر من 256 متغير خاص")?;
                write!(f, "{token}")
            }
            Self::TooManyUpvalues(token) => {
                writeln!(
                    f,
                    "لا يمكن لدالة أن تشير إلى أكثر من 256 متغير من دوال مغلقة عليها"
                )?;
                write!(f, "{token}")
            }
            Self::SameVarInScope(token) => {
                writeln!(f, "يوجد متغير يسمى \"{}\" في نفس المجموعة", token.lexeme())?;
                write!(f, "{token}")
            }
            Self::InvalidDes(token) => {
                writeln!(f, "يمكن فقط استخدام الكلمات والقوائم والكائنات في التوزيع")?;
                write!(f, "{token}")
            }
            Self::ReturnOutsideFunction(token) => {
                writeln!(f, "لا يمكن الإرجاع من خارج دالة")?;
                write!(f, "{token}")
            }
            Self::TooManyExports(token) => {
                writeln!(f, "لا يمكن تصدير أكثر من 65535 عنصر")?;
                write!(f, "{token}")
            }
            Self::OutsideLoopBreak(token) => {
                writeln!(f, "لا يمكن استخدام \"إكسر\" خارج حلقة تكرارية")?;
                write!(f, "{token}")
            }
            Self::OutsideLoopContinue(token) => {
                writeln!(f, "لا يمكن استخدام \"واصل\" خارج حلقة تكرارية")?;
                write!(f, "{token}")
            }
            Self::InvalidImportUsage(token) => {
                writeln!(f, "لا يمكن التصدير من داخل الدوال أو المجموعات")?;
                write!(f, "{token}")
            }
            Self::InvalidExportUsage(token) => {
                writeln!(f, "لا يمكن الاستيراد من داخل الدوال أو المجموعات")?;
                write!(f, "{token}")
            }
            Self::Io(token, err) => {
                writeln!(f, "{err}")?;
                write!(f, "{token}")
            }
            Self::ModuleParser(token, errors) => {
                writeln!(
                    f,
                    "{} أثناء تحليل الوحدة",
                    if errors.len() > 1 {
                        "حدث خطأ"
                    } else {
                        "حدثت بعض الأخطاء"
                    }
                )?;
                writeln!(f, "{token}")?;
                let mut iter = errors.iter();
                write!(f, "{}", iter.next().unwrap())?;
                while let Some(err) = iter.next() {
                    write!(f, "\n{err}")?;
                }
                Ok(())
            }
            Self::TooManyArgs(token) => {
                writeln!(f, "لا يمكن استدعاء دالة بأكثر من 255 مدخل")?;
                write!(f, "{token}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Type(Vec<DataType>, DataType, Rc<Token>, Backtrace),
    Name(String, Rc<Token>, Backtrace),
    AlreadyDefined(String, Rc<Token>, Backtrace),
    InvalidArgc(Arity, usize, Rc<Token>, Backtrace),
    InvalidIdx(Rc<Token>, Backtrace),
    OutOfRange(usize, usize, Rc<Token>, Backtrace),
    User(Value, Rc<Token>, Backtrace),
    ListUnpack(usize, usize, Rc<Token>, Backtrace),
    UndefinedKey(String, Rc<Token>, Backtrace),
    Io(Rc<io::Error>, Rc<Token>, Backtrace),
}

impl RuntimeError {
    pub fn msg(&self) -> String {
        match self {
            Self::Type(expected, received, ..) => {
                format!(
                    "توقعت {} ولكن حصلت على {received}",
                    expected
                        .iter()
                        .map(|dt| format!("{dt}"))
                        .collect::<Vec<_>>()
                        .join("أو ")
                )
            }
            Self::Name(name, ..) => format!("المتغير {name} غير معرّف"),
            Self::AlreadyDefined(name, ..) => format!("المتغير {name} معرّف مسبقاً"),
            Self::InvalidArgc(arity, argc, ..) => {
                let required = arity.required();
                let optional = arity.optional();
                let mut buf = String::from("عدد مدخلات خاطئ: توقعت ");
                match argc {
                    x if *x < required => buf += format!("على الأقل {required}").as_str(),
                    x if *x > required => {
                        buf += format!("على الأكثر {}", required + optional).as_str()
                    }
                    _ => {}
                }
                buf += format!(" ولكن حصلت على {argc}").as_str();
                buf
            }
            Self::InvalidIdx(..) => format!("يجب أن تكون القيمة المفهرس بها عدداً صحيحاً موجباً"),
            Self::OutOfRange(idx, len, ..) => {
                format!("لا يمكن الفهرسة ب{idx} في مرتّب حجمه {len}")
            }
            Self::User(value, ..) => format!("{value}"),
            Self::ListUnpack(to, len, ..) => {
                format!("لا يمكن توزيع قائمة حجمها {len} إلى عنصر {to}")
            }
            Self::UndefinedKey(key, ..) => format!("لا توجد الخاصية {key} في هذا الكائن"),
            Self::Io(err, ..) => format!("{err}"),
        }
    }

    pub fn backtrace(&self) -> &Backtrace {
        match self {
            Self::Type(.., backtrace)
            | Self::Name(.., backtrace)
            | Self::AlreadyDefined(.., backtrace)
            | Self::InvalidArgc(.., backtrace)
            | Self::InvalidIdx(.., backtrace)
            | Self::OutOfRange(.., backtrace)
            | Self::User(.., backtrace)
            | Self::ListUnpack(.., backtrace)
            | Self::UndefinedKey(.., backtrace)
            | Self::Io(.., backtrace) => backtrace,
        }
    }

    pub fn backtrace_mut(&mut self) -> &mut Backtrace {
        match self {
            Self::Type(.., backtrace)
            | Self::Name(.., backtrace)
            | Self::AlreadyDefined(.., backtrace)
            | Self::InvalidArgc(.., backtrace)
            | Self::InvalidIdx(.., backtrace)
            | Self::OutOfRange(.., backtrace)
            | Self::User(.., backtrace)
            | Self::ListUnpack(.., backtrace)
            | Self::UndefinedKey(.., backtrace)
            | Self::Io(.., backtrace) => backtrace,
        }
    }
}

impl TokenInside for RuntimeError {
    fn token(&self) -> Rc<Token> {
        match self {
            Self::Type(.., token, _)
            | Self::Name(.., token, _)
            | Self::AlreadyDefined(.., token, _)
            | Self::InvalidArgc(.., token, _)
            | Self::InvalidIdx(.., token, _)
            | Self::OutOfRange(.., token, _)
            | Self::User(.., token, _)
            | Self::ListUnpack(.., token, _)
            | Self::UndefinedKey(.., token, _)
            | Self::Io(.., token, _) => Rc::clone(token),
        }
    }
}

impl fmt::Display for RuntimeError {
    /// Expects `backtrace` to at least contain a single frame.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}\n{}\n{}",
            "خطأ تنفيذي: ".bright_red(),
            self.msg(),
            self.token(),
            self.backtrace(),
        )
    }
}

impl Into<Value> for RuntimeError {
    fn into(self) -> Value {
        match self {
            Self::User(value, ..) => value,
            err => Value::from(err.msg()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Backtrace {
    inner: Vec<(Option<String>, Rc<Token>)>,
}

impl Backtrace {
    pub fn push(&mut self, name: Option<String>, token: Rc<Token>) {
        self.inner.push((name, token));
    }
}

impl Default for Backtrace {
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! write_frame {
            ($frame:ident) => {{
                let (line, _) = $frame.1.pos();
                write!(
                    f,
                    "في {} السطر رقم {line}",
                    match &$frame.0 {
                        Some(name) => format!("الدالة {name}"),
                        None => "دالة غير معروفة".into(),
                    }
                )
            }};
        }
        let mut iter = self.inner.iter();
        if let Some(frame) = iter.next() {
            write_frame!(frame)?;
            while let Some(frame) = iter.next() {
                write!(f, "\n")?;
                write_frame!(frame)?
            }
        }
        Ok(())
    }
}
