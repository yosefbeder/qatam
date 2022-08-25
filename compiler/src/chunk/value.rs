use super::Chunk;
use std::convert::{From, Into, TryFrom};
use std::{cell::RefCell, cmp, collections::HashMap, fmt, fs, ops, rc::Rc};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Object(Object),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    Nil,
    Bool,
    Number,
    String,
    HashMap,
    List,
    File,
    Function,
    Closure,
    Native,
    Iterator,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Nil => "عدم",
                Self::Bool => "قيمة منطقية",
                Self::Number => "عدد",
                Self::String => "نص",
                Self::HashMap => "كائن",
                Self::List => "قائمة",
                Self::File => "ملف",
                Self::Function => "دالة",
                Self::Closure => "دالة",
                Self::Native => "دالة مدمجة",
                Self::Iterator => "مكرر",
            }
        )
    }
}

impl Value {
    /// `Nil`, `Bool(false)`, `Number(0)`, and empty sequences (i.e., empty strings, lists, hash maps) are falsy, the rest are truthy.
    pub fn truthy(&self) -> bool {
        match self {
            Self::Nil | Self::Bool(false) => false,
            Self::Number(number) if *number == 0.0 => false,
            Self::String(string) if string.len() == 0 => false,
            Self::Object(Object::List(list)) if list.borrow().len() == 0 => false,
            Self::Object(Object::HashMap(hash_map)) if hash_map.borrow().len() == 0 => false,
            _ => true,
        }
    }

    pub fn typ(&self) -> DataType {
        match self {
            Self::Nil => DataType::Nil,
            Self::Bool(..) => DataType::Bool,
            Self::Number(..) => DataType::Number,
            Self::String(..) => DataType::String,
            Self::Object(Object::HashMap(..)) => DataType::HashMap,
            Self::Object(Object::List(..)) => DataType::List,
            Self::Object(Object::File(..)) => DataType::File,
            Self::Object(Object::Function(..)) => DataType::Function,
            Self::Object(Object::Closure(..)) => DataType::Closure,
            Self::Object(Object::Native(..)) => DataType::Native,
            Self::Object(Object::Iterator(..)) => DataType::Iterator,
        }
    }
}

impl PartialEq for Value {
    /// Compares `Nil`, `Bool`, `Number`, and `String` by value and the rest by reference.
    ///
    /// Values of different types aren't equal.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(number) => Self::Number(-number),
            _ => unreachable!(),
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    /// Adds numbers and concatinates sequences.
    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::String(a), Self::String(b)) => Self::String(format!("{a}{b}")),
            (Self::Object(Object::List(a)), Self::Object(Object::List(b))) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                Self::from([a, b].concat())
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
        Self::Bool(!self.truthy())
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "عدم"),
            Self::Bool(value) => {
                if *value {
                    write!(f, "صحيح")
                } else {
                    write!(f, "خطأ")
                }
            }
            Self::Number(number) => write!(f, "{number}"),
            Self::String(string) => write!(f, "{string}"),
            Self::Object(object) => write!(f, "{object}"),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(number: f64) -> Self {
        Self::Number(number)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(hash_map: HashMap<String, Value>) -> Self {
        Self::Object(Object::HashMap(Rc::new(RefCell::new(hash_map))))
    }
}

impl From<Vec<Value>> for Value {
    fn from(list: Vec<Value>) -> Self {
        Self::Object(Object::List(Rc::new(RefCell::new(list))))
    }
}

impl From<File> for Value {
    fn from(file: File) -> Self {
        Self::Object(Object::File(Rc::new(RefCell::new(file))))
    }
}

impl From<Function> for Value {
    fn from(function: Function) -> Self {
        Self::Object(Object::Function(Rc::new(function)))
    }
}

impl From<Closure> for Value {
    fn from(closure: Closure) -> Self {
        Self::Object(Object::Closure(Rc::new(closure)))
    }
}

impl From<Native> for Value {
    fn from(native: Native) -> Self {
        Self::Object(Object::Native(native))
    }
}

impl From<Iterator> for Value {
    fn from(iterator: Iterator) -> Self {
        Self::Object(Object::Iterator(Rc::new(iterator)))
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    HashMap(Rc<RefCell<HashMap<String, Value>>>),
    List(Rc<RefCell<Vec<Value>>>),
    File(Rc<RefCell<File>>),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    Native(Native),
    Iterator(Rc<Iterator>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HashMap(a), Self::HashMap(b)) => Rc::ptr_eq(a, b),
            (Self::List(a), Self::List(b)) => Rc::ptr_eq(a, b),
            (Self::File(a), Self::File(b)) => Rc::ptr_eq(a, b),
            (Self::Function(a), Self::Function(b)) => Rc::ptr_eq(a, b),
            (Self::Closure(a), Self::Closure(b)) => Rc::ptr_eq(a, b),
            (Self::Native(a), Self::Native(b)) => *a as usize == *b as usize,
            (Self::Iterator(a), Self::Iterator(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashMap(hash_map) => {
                let tmp = hash_map.borrow();
                let mut iter = tmp.keys();
                write!(f, "{{")?;
                if let Some(key) = iter.next() {
                    write!(f, "{key}: {}", tmp.get(key).unwrap())?;
                    while let Some(key) = iter.next() {
                        write!(f, "{key}: {}", tmp.get(key).unwrap())?;
                    }
                }
                write!(f, "}}")
            }
            Self::List(list) => {
                let tmp = list.borrow();
                let mut iter = tmp.iter();
                write!(f, "[")?;
                if let Some(value) = iter.next() {
                    write!(f, "{value}")?;
                    while let Some(value) = iter.next() {
                        write!(f, "، {value}")?;
                    }
                }
                write!(f, "]")
            }
            Self::File(file) => write!(f, "{}", file.borrow()),
            Self::Function(function) => write!(f, "{function}"),
            Self::Closure(closure) => write!(f, "{}", closure.function),
            Self::Native(native) => write!(f, "<{native:?}دالة مدمجة مختزنة في >"),
            Self::Iterator(iterator) => write!(f, "{iterator}"),
        }
    }
}

#[derive(Debug)]
pub struct File {
    name: String,
    mode: FileMode,
    file: fs::File,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<ملف {} مفتوح على وضع {}>", self.name, self.mode)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileMode {
    Read,
    Write,
    All,
}

impl fmt::Display for FileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = (*self).into();
        write!(f, "{string}")
    }
}

const READ: &str = "قراءة";
const WRITE: &str = "كتابة";
const ALL: &str = "أي شئ";

impl TryFrom<String> for FileMode {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            READ => Ok(Self::Read),
            WRITE => Ok(Self::Write),
            ALL => Ok(Self::All),
            _ => Err(()),
        }
    }
}

impl Into<String> for FileMode {
    fn into(self) -> String {
        match self {
            Self::Read => READ.to_owned(),
            Self::Write => WRITE.to_owned(),
            Self::All => ALL.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: Option<String>,
    /// Consists of three main "subchunks":
    ///
    /// 1. Default values.
    /// 2. Variadic param builder which reduces all of the additional values on the stack to a single array (only for variadic functions).
    /// 3. Destructuring: and it destructures the arguments in a reversed order.
    /// 4. Body.
    chunk: Chunk,
    arity: Arity,
    /// Maps the number of optional parameters provided to the place in which the rest are written.
    defaults: Vec<usize>,
    /// Represents the `ip` of the first instruction in the variadic param builder (if the function is variadic) or the code for destructuring otherwise.
    body: usize,
}

impl Function {
    pub fn new(
        name: Option<String>,
        chunk: Chunk,
        arity: Arity,
        defaults: Vec<usize>,
        body: usize,
    ) -> Self {
        Self {
            name,
            chunk,
            arity,
            defaults,
            body,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<")?;
        match &self.name {
            Some(name) => write!(f, "دالة {name}")?,
            None => write!(f, "دالة غير مسماة")?,
        };
        write!(f, " المختزنة في {:?}>", self as *const Self)
    }
}

#[derive(Debug, Clone)]
pub struct Arity {
    typ: ArityType,
    required: usize,
    optional: usize,
}

impl Arity {
    pub fn new(typ: ArityType, required: usize, optional: usize) -> Self {
        Self {
            typ,
            required,
            optional,
        }
    }

    pub fn typ(&self) -> ArityType {
        self.typ
    }

    pub fn required(&self) -> usize {
        self.required
    }

    pub fn optional(&self) -> usize {
        self.optional
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArityType {
    Fixed,
    Variadic,
}

#[derive(Debug, Clone)]
pub enum Upvalue {
    Open(usize),
    Closed(Value),
}

#[derive(Debug)]
pub struct Closure {
    function: Rc<Function>,
    upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

impl Closure {
    /// Returns where the function should start executing giving `argc`.
    ///
    /// Fails if `argc` doesn't meet the requirements of `Arity`.
    pub fn start_ip(self, argc: usize) -> Result<usize, ()> {
        let Arity {
            typ,
            required,
            optional,
        } = self.function.arity.clone();
        match argc {
            x if x < required => Err(()),
            x if x >= required && x <= required + optional => {
                Ok(self.function.defaults[argc - required])
            }
            x if x >= required => {
                if typ == ArityType::Variadic {
                    Ok(self.function.body)
                } else {
                    Err(())
                }
            }
            _ => unreachable!(),
        }
    }
}

pub type Native = fn() -> Result<Value, Value>;

#[derive(Debug)]
pub struct Iterator {
    iterable: Iterable,
    counter: usize,
}

impl fmt::Display for Iterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<مكرر مختزن في {:?}>", self as *const Self)
    }
}

#[derive(Debug, Clone)]
pub enum Iterable {
    HashMap(Rc<RefCell<HashMap<String, Value>>>),
    List(Rc<RefCell<Vec<Value>>>),
    String(String),
}
