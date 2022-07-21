use super::{chunk::Chunk, vm::Frame};
use std::{cell::RefCell, cmp, collections::HashMap, convert::From, fmt, fs::File, ops, rc::Rc};

#[derive(Clone, Copy, PartialEq)]
pub enum ArityType {
    Fixed,
    Variadic,
}

#[derive(Clone)]
pub struct Arity {
    pub typ: ArityType,
    pub required: usize,
    pub optional: usize,
}

impl Arity {
    pub fn new(typ: ArityType, required: usize, optional: usize) -> Self {
        Self {
            typ,
            required,
            optional,
        }
    }
}

impl Default for Arity {
    fn default() -> Self {
        Self {
            typ: ArityType::Fixed,
            required: 0,
            optional: 0,
        }
    }
}

pub struct Function {
    name: Option<String>,
    chunk: Chunk,
    arity: Arity,
    defaults: Vec<usize>,
    start_ip: usize,
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
    pub fn new(
        name: Option<String>,
        chunk: Chunk,
        arity: Arity,
        defaults: Vec<usize>,
        start_ip: usize,
    ) -> Self {
        Self {
            name,
            chunk,
            arity,
            defaults,
            start_ip,
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
    function: Rc<Function>,
    up_values: Vec<Rc<RefCell<UpValue>>>,
}

impl Closure {
    pub fn new(function: Rc<Function>, up_values: Vec<Rc<RefCell<UpValue>>>) -> Self {
        Self {
            function,
            up_values,
        }
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.function.name
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.function.chunk
    }

    pub fn get_arity(&self) -> Arity {
        self.function.arity.clone()
    }

    pub fn get_defaults(&self) -> Vec<usize> {
        self.function.defaults.clone()
    }

    pub fn get_start_ip(&self) -> usize {
        self.function.start_ip
    }

    pub fn get_up_values(&self) -> &Vec<Rc<RefCell<UpValue>>> {
        &self.up_values
    }

    pub fn get_up_value(&self, idx: usize) -> Rc<RefCell<UpValue>> {
        Rc::clone(self.up_values.get(idx).unwrap())
    }
}

pub type Native = fn(&Frame, usize) -> Result<Value, Value>;

#[derive(Clone)]
pub enum Object {
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    Native(Native),
    File(Rc<RefCell<File>>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a == b,
            (Self::List(a), Self::List(b)) => Rc::ptr_eq(a, b),
            (Self::Object(a), Self::Object(b)) => Rc::ptr_eq(a, b),
            (Self::Function(a), Self::Function(b)) => Rc::ptr_eq(a, b),
            (Self::Closure(a), Self::Closure(b)) => Rc::ptr_eq(a, b),
            (Self::Native(a), Self::Native(b)) => *a as usize == *b as usize,
            (Self::File(a), Self::File(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Object(Object),
}

impl Value {
    pub fn new_string(string: String) -> Self {
        Self::Object(Object::String(string))
    }

    pub fn new_list(list: Vec<Value>) -> Self {
        Self::Object(Object::List(Rc::new(RefCell::new(list))))
    }

    pub fn new_object(object: HashMap<String, Value>) -> Self {
        Self::Object(Object::Object(Rc::new(RefCell::new(object))))
    }

    pub fn new_function(function: Function) -> Self {
        Self::Object(Object::Function(Rc::new(function)))
    }

    pub fn new_closure(function: Rc<Function>, up_values: Vec<Rc<RefCell<UpValue>>>) -> Self {
        Self::Object(Object::Closure(Rc::new(Closure::new(function, up_values))))
    }

    pub fn new_native(native: Native) -> Self {
        Self::Object(Object::Native(native))
    }

    pub fn new_file(file: File) -> Self {
        Self::Object(Object::File(Rc::new(RefCell::new(file))))
    }

    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Number(_) => "عدد",
            Value::Bool(_) => "ثنائي",
            Value::Nil => "عدم",
            Value::Object(obj) => match obj {
                Object::String(_) => "نص",
                Object::List(_) => "قائمة",
                Object::Object(_) => "كائن",
                Object::Function(_) => unreachable!(),
                Object::Closure(_) => "دالة",
                Object::Native(_) => "دالة مدمجة",
                Object::File(_) => "ملف",
            },
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
            Value::Object(Object::String(_)) => true,
            _ => false,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Value::Object(Object::String(string)) => string.clone(),
            _ => unreachable!(),
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Value::Number(n) => n.fract() == 0.0,
            _ => false,
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Value::Number(n) => *n as i32,
            _ => unreachable!(),
        }
    }

    pub fn as_function(&self) -> Rc<Function> {
        match self {
            Value::Object(Object::Function(function)) => Rc::clone(function),
            _ => unreachable!(),
        }
    }

    pub fn are_numbers(rhs: &Self, lhs: &Self) -> bool {
        match (rhs, lhs) {
            (Value::Number(_), Value::Number(_)) => true,
            _ => false,
        }
    }

    pub fn are_strings(rhs: &Self, lhs: &Self) -> bool {
        match (rhs, lhs) {
            (Value::Object(Object::String(_)), Value::Object(Object::String(_))) => true,
            _ => false,
        }
    }

    pub fn are_addable(rhs: &Self, lhs: &Self) -> bool {
        Self::are_numbers(rhs, lhs) || Self::are_strings(rhs, lhs)
    }

    pub fn are_subtractable(rhs: &Self, lhs: &Self) -> bool {
        Self::are_numbers(rhs, lhs)
    }

    pub fn are_multipliable(rhs: &Self, lhs: &Self) -> bool {
        Self::are_numbers(rhs, lhs)
    }

    pub fn are_dividable(rhs: &Self, lhs: &Self) -> bool {
        Self::are_numbers(rhs, lhs)
    }

    pub fn are_remainderable(rhs: &Self, lhs: &Self) -> bool {
        Self::are_numbers(rhs, lhs)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::new_string(string)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Number(n) => format!("{}", n),
                Self::Bool(val) => format!("{}", if *val { "صحيح" } else { "خطأ" }),
                Self::Nil => format!("عدم"),
                Self::Object(obj) => {
                    match obj {
                        Object::String(string) => format!("{}", string),
                        Object::List(items) => {
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
                        Object::Object(items) => {
                            let mut buffer = String::from("{");
                            let items = items.borrow();
                            let mut entries = items.iter().collect::<Vec<_>>();
                            entries.sort_by(|a, b| a.0.cmp(&b.0));
                            match entries.iter().nth(0) {
                                Some((key, value)) => {
                                    buffer += format!("{key}: {value}").as_str();
                                    for (key, value) in entries.iter().skip(1) {
                                        buffer += format!("، {key}: {value}").as_str();
                                    }
                                }
                                None => {}
                            }

                            buffer += "}";
                            buffer
                        }
                        Object::Function(function) => format!("{}", function),
                        Object::Closure(closure) => format!("{}", closure.function),
                        Object::Native(_) => format!("<دالة مدمجة>"),
                        Object::File(file) => format!("<ملف {:?}>", file.borrow()),
                    }
                }
            }
        )
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
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
            (Self::Object(Object::String(a)), Self::Object(Object::String(b))) => {
                Self::new_string(format!("{a}{b}"))
            }
            _ => unreachable!(),
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
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            (Self::Object(a), Self::Object(b)) => a == b,
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
