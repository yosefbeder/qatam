use super::chunk::Chunk;
// use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub struct Function<'a> {
    name: Option<String>,
    chunk: Chunk<'a>,
    arity: u8,
}

#[cfg(feature = "debug-bytecode")]
impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();

        match &self.name {
            Some(name) => buffer += format!("=== دالة {} ===\n", name).as_str(),
            None => buffer += format!("=== دالة غير معروفة ===\n").as_str(),
        }

        buffer += format!("{:?}", self.chunk).as_str();

        buffer += "أنهي\n";

        write!(f, "{}", buffer)
    }
}

impl<'a> Function<'a> {
    pub fn new(name: Option<String>, chunk: Chunk<'a>, arity: u8) -> Self {
        Self { name, chunk, arity }
    }
}

#[derive(Clone)]
pub enum Value<'a> {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    // List(Vec<Value<'a>>),
    // Object(HashMap<String, Value<'a>>),
    Function(Rc<Function<'a>>),
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Number(n) => format!("{}", n),
                Self::String(string) => format!("{}", string),
                Self::Bool(val) => format!("{}", if *val { "صحيح" } else { "خطأ" }),
                Self::Nil => format!("عدم"),
                Self::Function(function) => match &function.name {
                    Some(name) => format!("<دالة {}>", name),
                    None => format!("<دالة غير معروفة>"),
                },
            }
        )
    }
}
