use super::{chunk::Chunk, vm::Vm};
use std::{
    cell::RefCell, cmp, collections::HashMap, convert::TryInto, fmt, fs::File, ops, path::PathBuf,
    rc::Rc,
};

#[derive(Clone, Copy)]
pub enum Arity {
    Fixed(u8),
    Variadic(u8),
}

pub struct Function {
    name: Option<String>,
    pub chunk: Chunk,
    pub arity: Arity,
    pub path: Option<PathBuf>,
}

impl fmt::Debug for Function {
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

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "<دالة {}>", name),
            None => write!(f, "<دالة غير معروفة>"),
        }
    }
}

impl Function {
    pub fn new(name: Option<String>, chunk: Chunk, arity: Arity, path: Option<PathBuf>) -> Self {
        Self {
            name,
            chunk,
            arity,
            path: match path {
                Some(path) => Some(path.to_owned()),
                None => None,
            },
        }
    }
}

#[derive(Clone)]
pub enum UpValue {
    Open(usize),
    Closed(Value),
}

impl UpValue {
    pub fn new(idx: usize) -> Self {
        Self::Open(idx)
    }

    pub fn close(&mut self, value: Value) {
        *self = UpValue::Closed(value);
    }

    pub fn is_open(&self) -> bool {
        match self {
            Self::Open(_) => true,
            _ => false,
        }
    }

    pub fn as_open(&self) -> usize {
        match self {
            UpValue::Open(idx) => *idx,
            UpValue::Closed(_) => unreachable!(),
        }
    }
}

pub struct Closure {
    pub function: Rc<Function>,
    pub up_values: Vec<Rc<RefCell<UpValue>>>,
}

impl Closure {
    pub fn new(function: Rc<Function>, up_values: Vec<Rc<RefCell<UpValue>>>) -> Self {
        Self {
            function,
            up_values,
        }
    }
}

pub type Native = fn(&Vm, usize) -> Result<Value, String>;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    Native(Native),
    File(Rc<RefCell<File>>),
}

impl Value {
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Number(_) => "عدد",
            Value::String(_) => "نص",
            Value::Bool(_) => "ثنائي",
            Value::Nil => "عدم",
            Value::List(_) => "قائمة",
            Value::Object(_) => "كائن",
            Value::Function(_) => unreachable!(),
            Value::Closure(_) => "دالة",
            Value::Native(_) => "دالة مدمجة",
            Value::File(_) => "ملف",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn as_function(&self) -> Rc<Function> {
        match self {
            Value::Function(f) => Rc::clone(f),
            _ => unreachable!(),
        }
    }

    pub fn are_numbers(right: &Self, left: &Self) -> bool {
        match (right, left) {
            (Value::Number(_), Value::Number(_)) => true,
            _ => false,
        }
    }

    pub fn are_subtractable(right: &Self, left: &Self) -> bool {
        Self::are_numbers(right, left)
    }

    pub fn are_multipliable(right: &Self, left: &Self) -> bool {
        Self::are_numbers(right, left)
    }

    pub fn are_dividable(right: &Self, left: &Self) -> bool {
        Self::are_numbers(right, left)
    }

    pub fn are_remainderable(right: &Self, left: &Self) -> bool {
        Self::are_numbers(right, left)
    }
}

impl TryInto<isize> for Value {
    type Error = ();

    fn try_into(self) -> Result<isize, Self::Error> {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(n as isize)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Number(n) => format!("{}", n),
                Self::String(string) => format!("{}", string),
                Self::Bool(val) => format!("{}", if *val { "صحيح" } else { "خطأ" }),
                Self::Nil => format!("عدم"),
                Self::List(items) => {
                    let mut buffer = String::from("[");

                    match items.borrow().get(0) {
                        Some(item) => {
                            buffer += format!("{}", item).as_str();
                            for item in items.borrow().iter().skip(1) {
                                buffer += format!("، {}", item).as_str();
                            }
                        }
                        None => {}
                    }

                    buffer += "]";
                    buffer
                }
                Self::Object(items) => {
                    let mut buffer = String::from("{");

                    match items.borrow().iter().nth(0) {
                        Some((key, value)) => {
                            buffer += format!("{key}: {value}،").as_str();
                            for (key, value) in items.borrow().iter().skip(1) {
                                buffer += format!("، {key}: {value}").as_str();
                            }
                        }
                        None => {}
                    }

                    buffer += "}";
                    buffer
                }
                Self::Function(function) => format!("{}", function),
                Self::Closure(closure) => format!("{}", closure.function),
                Self::Native(_) => format!("<دالة مدمجة>"),
                Self::File(file) => format!("<ملف {:?}>", file.borrow()),
            }
        )
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(a) => Self::Number(-a),
            _ => unreachable!(),
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (&self, &other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            _ => Self::String(format!("{}{}", self, other)),
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            _ => unreachable!(),
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            //TODO consider adding support for strings (with numbers)
            _ => unreachable!(),
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a / b),
            _ => unreachable!(),
        }
    }
}

impl ops::Rem for Value {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a % b),
            _ => unreachable!(),
        }
    }
}

impl ops::Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Bool(!self.is_truthy())
    }
}

impl cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
