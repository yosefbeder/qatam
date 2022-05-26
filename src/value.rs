use super::{chunk::Chunk, TAB_SIZE};
use std::collections::HashMap;
use std::{cell::RefCell, cmp, fmt, ops, rc::Rc};

pub struct Function<'a> {
    name: Option<String>,
    pub chunk: Chunk<'a>,
    pub arity: u8, //TODO make it optional
}

#[cfg(feature = "debug-execution")]
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

impl fmt::Display for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "<دالة {}>", name),
            None => write!(f, "<دالة غير معروفة>"),
        }
    }
}

impl<'a> Function<'a> {
    pub fn new(name: Option<String>, chunk: Chunk<'a>, arity: u8) -> Self {
        Self { name, chunk, arity }
    }
}

#[derive(Clone)]
pub enum UpValue<'a> {
    Open(usize),
    Closed(Value<'a>),
}

impl<'a> UpValue<'a> {
    pub fn new(idx: usize) -> Self {
        Self::Open(idx)
    }

    pub fn close(&mut self, value: Value<'a>) {
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

pub struct Closure<'a> {
    pub function: Rc<Function<'a>>,
    pub up_values: Vec<Rc<RefCell<UpValue<'a>>>>,
}

impl<'a> Closure<'a> {
    pub fn new(function: Rc<Function<'a>>, up_values: Vec<Rc<RefCell<UpValue<'a>>>>) -> Self {
        Self {
            function,
            up_values,
        }
    }
}

pub struct NFunction<'a> {
    pub function: fn(Vec<Value<'a>>) -> Value<'a>,
    pub arity: Option<u8>,
}

impl<'a> NFunction<'a> {
    pub fn new(function: fn(Vec<Value<'a>>) -> Value, arity: Option<u8>) -> Self {
        NFunction { function, arity }
    }
}

#[derive(Clone)]
pub enum Value<'a> {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    List(Rc<RefCell<Vec<Value<'a>>>>),
    Object(Rc<RefCell<HashMap<String, Value<'a>>>>),
    Function(Rc<Function<'a>>),
    Closure(Rc<Closure<'a>>),
    NFunction(Rc<NFunction<'a>>),
}

impl<'a> Value<'a> {
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

    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            _ => unreachable!(),
        }
    }

    pub fn as_function(&self) -> Rc<Function<'a>> {
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
                Self::List(items) => {
                    let mut buffer = String::from("[");

                    if items.borrow().len() != 0 {
                        buffer += "\n";
                        for item in items.borrow().iter() {
                            buffer += " ".repeat(TAB_SIZE).as_str();
                            buffer += format!("{}،\n", item).as_str();
                        }
                    }

                    buffer += "]";
                    buffer
                }
                Self::Object(items) => {
                    let mut buffer = String::from("{");

                    if !items.borrow().is_empty() {
                        buffer += "\n";
                        for (key, value) in items.borrow().iter() {
                            buffer += " ".repeat(TAB_SIZE).as_str();
                            buffer += format!("{}: {}،\n", key, value).as_str();
                        }
                    }

                    buffer += "}";
                    buffer
                }
                Self::Function(function) => format!("{}", function),
                Self::Closure(closure) => format!("{}", closure.function),
                Self::NFunction(_) => format!("<دالة محلية>"),
            }
        )
    }
}

impl<'a> ops::Neg for Value<'a> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(a) => Self::Number(-a),
            _ => unreachable!(),
        }
    }
}

impl<'a> ops::Add for Value<'a> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (&self, &other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            _ => Self::String(format!("{}{}", self, other)),
        }
    }
}

impl<'a> ops::Sub for Value<'a> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            _ => unreachable!(),
        }
    }
}

impl<'a> ops::Mul for Value<'a> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            //TODO consider adding support for strings (with numbers)
            _ => unreachable!(),
        }
    }
}

impl<'a> ops::Div for Value<'a> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a / b),
            _ => unreachable!(),
        }
    }
}

impl<'a> ops::Rem for Value<'a> {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a % b),
            _ => unreachable!(),
        }
    }
}

impl<'a> ops::Not for Value<'a> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Bool(!self.is_truthy())
    }
}

impl<'a> cmp::PartialEq for Value<'a> {
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

impl<'a> cmp::PartialOrd for Value<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
