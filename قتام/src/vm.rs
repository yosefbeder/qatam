use super::{
    chunk::Instruction::{self, *},
    natives::NATIVES,
    token::Token,
    utils::combine,
    value::{Arity, ArityType::*, Closure, Function, Native, Object, UpValue, Value},
};
use std::{
    cell::RefCell, collections::HashMap, fmt, fs::File, path::PathBuf, rc::Rc, time::SystemTime,
};

#[derive(Debug, Clone)]
struct BacktraceFrame {
    token: Rc<Token>,
    name: Option<String>,
}

impl BacktraceFrame {
    fn new(frame: &Frame) -> Self {
        Self {
            token: frame.cur_token(),
            name: frame.get_closure().get_name().clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct Backtrace {
    frames: Vec<BacktraceFrame>,
}

impl Backtrace {
    fn new() -> Self {
        Self { frames: vec![] }
    }

    fn push(&mut self, frame: BacktraceFrame) {
        self.frames.push(frame);
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for frame in &self.frames {
            writeln!(
                f,
                "من {} {}",
                match &frame.name {
                    Some(name) => name,
                    None => "دالة غير معروفة",
                },
                frame.token.get_pos(),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct RuntimeError {
    value: Value,
    backtrace: Backtrace,
}

impl RuntimeError {
    fn new(value: Value) -> Self {
        RuntimeError {
            value,
            backtrace: Backtrace::new(),
        }
    }

    fn push_frame(&mut self, frame: &Frame) {
        self.backtrace.push(BacktraceFrame::new(frame))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "خطأ تنفيذي: {}", self.value.to_string())?;
        write!(f, "{}", self.backtrace)
    }
}

pub struct Vm {
    stack: Vec<Value>,
    tmps: Vec<Value>,
    globals: HashMap<String, Value>,
    open_up_values: Vec<Rc<RefCell<UpValue>>>,
    created_at: SystemTime,
    untrusted: bool,
}

impl Vm {
    pub fn new(untrusted: bool) -> Self {
        let mut vm = Self {
            stack: vec![],
            tmps: vec![],
            globals: HashMap::new(),
            open_up_values: vec![],
            created_at: SystemTime::now(),
            untrusted,
        };

        NATIVES.iter().for_each(|(name, native)| {
            vm.globals
                .insert(name.to_string(), Value::new_native(native.clone()));
        });

        vm
    }

    fn get_up_value(&self, idx: usize) -> Option<Rc<RefCell<UpValue>>> {
        self.open_up_values
            .iter()
            .find(|up_value| up_value.borrow().as_open() == idx)
            .cloned()
    }

    fn append_up_value(&mut self, idx: usize) -> Rc<RefCell<UpValue>> {
        let up_value = Rc::new(RefCell::new(UpValue::new(idx)));
        self.open_up_values.push(Rc::clone(&up_value));
        up_value
    }

    fn close_up_values(&mut self, location: usize) {
        let mut new = vec![];

        for up_value in self.open_up_values.iter() {
            let idx;

            match &*up_value.borrow() {
                UpValue::Open(idx_) => idx = *idx_,
                UpValue::Closed(_) => unreachable!(),
            }

            if idx >= location {
                up_value
                    .borrow_mut()
                    .close(self.stack.get(idx).unwrap().clone());
            } else {
                new.push(up_value.clone());
            }
        }
        self.open_up_values = new;
    }

    pub fn run(&mut self, function: Function) -> Result<(), ()> {
        if cfg!(feature = "debug-execution") {
            println!("---");
            println!("[DEBUG] started executing");
            println!("---");
        }

        let closure = Rc::new(Closure::new(Rc::new(function), vec![]));
        self.stack
            .push(Value::Object(Object::Closure(Rc::clone(&closure))));
        match Frame::new_closure(self, closure, 0, 0, None).run(0) {
            Err(err) => {
                eprint!("{err}")
            }
            _ => {}
        };
        self.stack.pop();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    slots: usize,
    ip: usize,
}

impl Handler {
    fn new(slots: usize, ip: usize) -> Self {
        Self { slots, ip }
    }
}

pub enum Frame<'a, 'b> {
    Closure {
        state: &'a mut Vm,
        closure: Rc<Closure>,
        ip: usize,
        slots: usize,
        enclosing_up_values: Option<&'b Vec<Rc<RefCell<UpValue>>>>,
        handlers: Vec<Handler>,
    },
    Native {
        state: &'a mut Vm,
        native: Native,
        slots: usize,
    },
}

impl<'a, 'b> Frame<'a, 'b> {
    fn new_closure(
        state: &'a mut Vm,
        closure: Rc<Closure>,
        ip: usize,
        slots: usize,
        enclosing_up_values: Option<&'b Vec<Rc<RefCell<UpValue>>>>,
    ) -> Self {
        Self::Closure {
            state,
            closure,
            ip,
            slots,
            enclosing_up_values,
            handlers: vec![],
        }
    }

    fn new_native(state: &'a mut Vm, native: Native, slots: usize) -> Self {
        Self::Native {
            state,
            native,
            slots,
        }
    }

    fn is_native(&self) -> bool {
        match self {
            Self::Native { .. } => true,
            _ => false,
        }
    }

    fn get_closure(&self) -> &Rc<Closure> {
        match self {
            Self::Closure { closure, .. } => closure,
            Self::Native { .. } => unreachable!(),
        }
    }

    fn get_ip(&self) -> usize {
        match self {
            Self::Closure { ip, .. } => *ip,
            Self::Native { .. } => unreachable!(),
        }
    }

    fn set_ip(&mut self, next: usize) {
        match self {
            Self::Closure { ip, .. } => *ip = next,
            Self::Native { .. } => unreachable!(),
        }
    }

    fn get_enclosing_up_values(&self) -> &Option<&Vec<Rc<RefCell<UpValue>>>> {
        match self {
            Self::Closure {
                enclosing_up_values,
                ..
            } => enclosing_up_values,
            Self::Native { .. } => unreachable!(),
        }
    }

    fn get_handlers_mut(&mut self) -> &mut Vec<Handler> {
        match self {
            Self::Closure { handlers, .. } => handlers,
            Self::Native { .. } => unreachable!(),
        }
    }

    fn get_state_mut(&mut self) -> &mut Vm {
        match self {
            Self::Closure { state, .. } => state,
            Self::Native { state, .. } => state,
        }
    }

    fn get_state(&self) -> &Vm {
        match self {
            Self::Closure { state, .. } => state,
            Self::Native { state, .. } => state,
        }
    }

    fn get_native(&self) -> Native {
        match self {
            Self::Closure { .. } => unreachable!(),
            Self::Native { native, .. } => native.clone(),
        }
    }

    fn get_slots(&self) -> usize {
        match self {
            Self::Closure { slots, .. } => *slots,
            Self::Native { slots, .. } => *slots,
        }
    }

    fn read_byte(&self) -> usize {
        self.get_closure()
            .get_chunk()
            .get_byte(self.get_ip() + 1)
            .unwrap() as usize
    }

    fn read_up_value(&self, offset: usize) -> (bool, usize) {
        (
            self.get_closure().get_chunk().get_byte(offset).unwrap() != 0,
            self.get_closure().get_chunk().get_byte(offset + 1).unwrap() as usize,
        )
    }

    fn read_2bytes(&self) -> usize {
        combine(
            self.get_closure()
                .get_chunk()
                .get_byte(self.get_ip() + 1)
                .unwrap(),
            self.get_closure()
                .get_chunk()
                .get_byte(self.get_ip() + 2)
                .unwrap(),
        ) as usize
    }

    fn cur_byte(&self) -> Option<u8> {
        self.get_closure().get_chunk().get_byte(self.get_ip())
    }

    fn cur_instr(&self) -> Option<Instruction> {
        Some(self.cur_byte()?.into())
    }

    fn cur_token(&self) -> Rc<Token> {
        self.get_closure().get_chunk().get_token(self.get_ip())
    }

    fn pop(&mut self) -> Value {
        self.get_state_mut().stack.pop().unwrap()
    }

    fn push(&mut self, value: Value) {
        self.get_state_mut().stack.push(value);
    }

    fn last(&self) -> &Value {
        self.get_state().stack.last().unwrap()
    }

    fn get(&self, idx: usize) -> &Value {
        &self.get_state().stack[idx]
    }

    fn get_mut(&mut self, idx: usize) -> &mut Value {
        &mut self.get_state_mut().stack[idx]
    }

    fn truncate(&mut self, len: usize) {
        self.get_state_mut().close_up_values(len);
        self.get_state_mut().stack.truncate(len);
    }

    //>> Native functions utilities
    pub fn check_arity(arity: Arity, argc: usize) -> Result<(), Value> {
        if argc < arity.required {
            Err(Value::new_string(format!(
                " توقعت على الأقل عدد {} من المدخلات، ولكن حصلت على {} بدلاً من ذللك",
                arity.required, argc
            )))
        } else if arity.typ == Fixed && argc > arity.required + arity.optional {
            Err(Value::new_string(format!(
                "توقعت على الأكثر عدد {} من المدخلات، ولكن حصلت على {} بدلاً من ذلك",
                arity.required + arity.optional,
                argc
            )))
        } else {
            Ok(())
        }
    }

    pub fn check_trust(&self) -> Result<(), Value> {
        if self.get_state().untrusted {
            Err(Value::new_string(
                "لا يمكن تشغيل هذه الدالة على وضع عدم الثقة".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn nth(&self, idx: usize) -> &Value {
        self.get(self.get_slots() + idx)
    }

    pub fn nth_f64(&self, idx: usize) -> Result<f64, Value> {
        if let Value::Number(n) = self.nth(idx) {
            Ok(*n)
        } else {
            Err(format!("يجب أن يكون المدخل {idx} عدداً").into())
        }
    }

    pub fn nth_i32(&self, idx: usize) -> Result<i32, Value> {
        if let Value::Number(n) = self.nth(idx) {
            if n.fract() == 0.0 {
                Ok(*n as i32)
            } else {
                Err(format!("يجب أن يكون المدخل {idx} عدداً صحيحاً").into())
            }
        } else {
            Err(format!("يجب أن يكون المدخل {idx} عدداً صحيحاً").into())
        }
    }

    pub fn nth_u32(&self, idx: usize) -> Result<u32, Value> {
        if let Value::Number(n) = self.nth(idx) {
            if n.fract() == 0.0 && *n > 0.0 {
                Ok(*n as u32)
            } else {
                Err(format!("يجب أن يكون المدخل {idx} عدداً صحيحاً موجباً").into())
            }
        } else {
            Err(format!("يجب أن يكون المدخل {idx} عدداً صحيحاً موجباً").into())
        }
    }

    pub fn nth_string(&self, idx: usize) -> Result<&str, Value> {
        if let Value::Object(Object::String(string)) = self.nth(idx) {
            Ok(string)
        } else {
            Err(format!("يجب أن يكون المدخل {idx} نص").into())
        }
    }

    pub fn nth_char(&self, idx: usize) -> Result<char, Value> {
        if let Value::Object(Object::String(string)) = self.nth(idx) {
            if string.chars().count() == 1 {
                Ok(string.chars().nth(0).unwrap())
            } else {
                Err(format!("يجب أن يكون المدخل {idx} نص ذي حرف واحد").into())
            }
        } else {
            Err(format!("يجب أن يكون المدخل {idx} نص ذي حرف واحد").into())
        }
    }

    pub fn nth_object(&self, idx: usize) -> Result<&Rc<RefCell<HashMap<String, Value>>>, Value> {
        if let Value::Object(Object::Object(items)) = self.nth(idx) {
            Ok(items)
        } else {
            Err(format!("يجب أن يكون المدخل {idx} مجموعة").into())
        }
    }

    pub fn nth_list(&self, idx: usize) -> Result<&Rc<RefCell<Vec<Value>>>, Value> {
        if let Value::Object(Object::List(items)) = self.nth(idx) {
            Ok(items)
        } else {
            Err(format!("يجب أن يكون المدخل {idx} قائمة").into())
        }
    }

    pub fn nth_file(&self, idx: usize) -> Result<&Rc<RefCell<File>>, Value> {
        if let Value::Object(Object::File(path)) = self.nth(idx) {
            Ok(path)
        } else {
            Err(format!("يجب أن يكون المدخل {idx} ملف").into())
        }
    }

    pub fn nth_path(&self, idx: usize) -> Result<PathBuf, Value> {
        let path = self.nth_string(idx)?;
        Ok(PathBuf::from(path))
    }

    pub fn get_creation_time(&self) -> &SystemTime {
        &self.get_state().created_at
    }
    //<<

    fn string_to_err(&self, string: String) -> RuntimeError {
        let mut err = RuntimeError::new(Value::new_string(string));
        err.push_frame(self);
        err
    }

    fn run(&mut self, argc: usize) -> Result<Option<Value>, RuntimeError> {
        if self.is_native() {
            let returned =
                self.get_native()(self, argc).map_err(|value| RuntimeError::new(value))?;
            return Ok(Some(returned));
        }

        fn get_absolute_idx(idx: i32, len: usize) -> Result<usize, ()> {
            if idx >= 0 {
                if idx >= len as i32 {
                    return Err(());
                }
                Ok(idx as usize)
            } else {
                if -idx > len as i32 {
                    return Err(());
                }
                Ok((len as i32 + idx) as usize)
            }
        }

        while let Some(instr) = self.cur_instr() {
            if cfg!(feature = "debug-execution") {
                print!(
                    "{}",
                    self.get_closure()
                        .get_chunk()
                        .disassemble_instr_at(self.get_ip(), false)
                        .0
                );
            }

            let mut progress = 1i32;

            match instr {
                Pop => {
                    self.pop();
                }
                Constant8 => {
                    let idx = self.read_byte();
                    self.push(self.get_closure().get_chunk().get_constant(idx));
                    progress = 2;
                }
                Constant16 => {
                    let idx = self.read_2bytes();
                    self.push(self.get_closure().get_chunk().get_constant(idx));
                    progress = 3;
                }
                Negate => {
                    let popped = self.pop();
                    if !popped.is_number() {
                        return Err(self.string_to_err("يجب أن يكون المعامل رقماً".to_string()));
                    }
                    self.push(-popped);
                }
                Add => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_addable(&a, &b) {
                        return Err(self.string_to_err("لا يقبل المعاملان الجمع".to_string()));
                    }
                    self.push(a + b);
                }
                Subtract => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_subtractable(&a, &b) {
                        return Err(
                            self.string_to_err("لا يقبل المعاملان الطرح من بعضهما".to_string())
                        );
                    }
                    self.push(a - b);
                }
                Multiply => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_multipliable(&a, &b) {
                        return Err(
                            self.string_to_err("لا يقبل المعاملان الضرب في بعضهما".to_string())
                        );
                    }
                    self.push(a * b);
                }
                Divide => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_dividable(&a, &b) {
                        return Err(
                            self.string_to_err("لا يقبل المعاملان القسمة على بعضهما".to_string())
                        );
                    }
                    self.push(a / b);
                }
                Remainder => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_remainderable(&a, &b) {
                        return Err(
                            self.string_to_err("لا يقبل المعاملان القسمة على بعضهما".to_string())
                        );
                    }
                    self.push(a % b);
                }
                Not => {
                    let popped = self.pop();
                    self.push(!popped);
                }
                Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Greater => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_numbers(&a, &b) {
                        return Err(self.string_to_err("يجب أن يكون المعاملان أرقاماً".to_string()));
                    }
                    self.push(Value::Bool(a > b));
                }
                GreaterEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_numbers(&a, &b) {
                        return Err(self.string_to_err("يجب أن يكون المعاملان أرقاماً".to_string()));
                    }
                    self.push(Value::Bool(a >= b));
                }
                Less => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_numbers(&a, &b) {
                        return Err(self.string_to_err("يجب أن يكون المعاملان أرقاماً".to_string()));
                    }
                    self.push(Value::Bool(a < b));
                }
                LessEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    if !Value::are_numbers(&a, &b) {
                        return Err(self.string_to_err("يجب أن يكون المعاملان أرقاماً".to_string()));
                    }
                    self.push(Value::Bool(a <= b));
                }
                Jump => {
                    progress = self.read_2bytes() as i32;
                }
                JumpIfFalse => {
                    if self.last().is_truthy() {
                        progress = 3;
                    } else {
                        progress = self.read_2bytes() as i32;
                    }
                }
                JumpIfTrue => {
                    if !self.last().is_truthy() {
                        progress = 3;
                    } else {
                        progress = self.read_2bytes() as i32;
                    }
                }
                Loop => {
                    progress = -(self.read_2bytes() as i32);
                }
                DefineGlobal => {
                    let name = self.pop().to_string();
                    let value = self.pop();
                    if self.get_state().globals.contains_key(&name) {
                        return Err(self.string_to_err("يوجد متغير بهذا الاسم".to_string()));
                    }
                    self.get_state_mut().globals.insert(name.clone(), value);
                }
                SetGlobal => {
                    let name = self.pop().to_string();
                    let value = self.last().clone();
                    if !self.get_state().globals.contains_key(&name) {
                        return Err(self.string_to_err("لا يوجد متغير بهذا الاسم".to_string()));
                    }
                    self.get_state_mut().globals.insert(name, value);
                }
                GetGlobal => {
                    let name = self.pop().to_string();
                    if !self.get_state().globals.contains_key(&name) {
                        return Err(self.string_to_err("لا يوجد متغير بهذا الاسم".to_string()));
                    }
                    self.push(self.get_state().globals.get(&name).unwrap().clone());
                }
                GetLocal => {
                    let idx = self.get_slots() + self.read_byte();
                    self.push(self.get(idx).clone());
                    progress = 2;
                }
                SetLocal => {
                    let idx = self.get_slots() + self.read_byte();
                    *self.get_mut(idx) = self.last().clone();
                    progress = 2;
                }
                BuildList => {
                    let size = self.read_byte();
                    let mut items = vec![];
                    let len = self.get_state().stack.len();
                    for item in self.get_state_mut().stack.drain(len - size..) {
                        items.push(item);
                    }
                    self.push(Value::new_list(items));
                    progress = 2;
                }
                BuildObject => {
                    let size = self.read_byte();
                    let mut items = HashMap::new();
                    for _ in 0..size {
                        let value = self.pop();
                        let name = self.pop().to_string();
                        items.insert(name, value);
                    }
                    self.push(Value::new_object(items));
                    progress = 2;
                }
                Get => {
                    let key = self.pop();
                    let obj = self.pop();
                    match obj {
                        Value::Object(Object::Object(items)) => {
                            if !key.is_string() {
                                return Err(
                                    self.string_to_err("يجب أن يكون اسم الخاصية نصاً".to_string())
                                );
                            }
                            if let Some(value) = items.borrow().get(&key.to_string()) {
                                self.push(value.clone());
                            } else {
                                return Err(
                                    self.string_to_err("لا يوجد قيمة بهذا الاسم".to_string())
                                );
                            }
                        }
                        Value::Object(Object::List(items)) => {
                            if !key.is_int() {
                                return Err(self.string_to_err(
                                    "يجب أن يكون رقم العنصر عدداً صحيحاً".to_string(),
                                ));
                            }
                            let idx = get_absolute_idx(key.as_int(), items.borrow().len())
                                .map_err(|_| {
                                    self.string_to_err("لا يوجد عنصر بهذا الرقم".to_string())
                                })?;
                            self.push(items.borrow()[idx].clone());
                        }
                        Value::Object(Object::String(string)) => {
                            if !key.is_int() {
                                return Err(self.string_to_err(
                                    "يجب أن يكون رقم العنصر عدداً صحيحاً".to_string(),
                                ));
                            }
                            let idx = get_absolute_idx(key.as_int(), string.chars().count())
                                .map_err(|_| {
                                    self.string_to_err("لا يوجد عنصر بهذا الرقم".to_string())
                                })?;
                            self.push(Value::new_string(
                                string.chars().nth(idx).unwrap().to_string(),
                            ));
                        }
                        _ => {
                            return Err(self.string_to_err(
                                "يجب أن يكون المتغير نص أو قائمة أو كائن".to_string(),
                            ))
                        }
                    }
                }
                Set => {
                    let key = self.pop();
                    let obj = self.pop();
                    match obj {
                        Value::Object(Object::Object(items)) => {
                            if !key.is_string() {
                                return Err(
                                    self.string_to_err("يجب أن يكون اسم الخاصية نصاً".to_string())
                                );
                            }
                            items
                                .borrow_mut()
                                .insert(key.as_string(), self.last().clone());
                        }
                        Value::Object(Object::List(items)) => {
                            if !key.is_int() {
                                return Err(self.string_to_err(
                                    "يجب أن يكون رقم العنصر عدداً صحيحاً".to_string(),
                                ));
                            }

                            let idx = get_absolute_idx(key.as_int(), items.borrow().len())
                                .map_err(|_| {
                                    self.string_to_err("لا يوجد عنصر بهذا الرقم".to_string())
                                })?;

                            items.borrow_mut()[idx] = self.last().clone();
                        }
                        _ => {
                            return Err(
                                self.string_to_err("يجب أن يكون المتغير قائمة أو كائن".to_string())
                            )
                        }
                    }
                }
                Closure => {
                    //TODO test
                    let count = self.read_byte() as usize;
                    let function = self.pop().as_function();
                    let up_values = {
                        let mut data = Vec::with_capacity(count);
                        for idx in 0..count {
                            let offset = self.get_ip() + 2 + idx * 2;
                            data.push(self.read_up_value(offset))
                        }

                        let mut res = Vec::with_capacity(count);

                        for (is_local, mut idx) in data {
                            if is_local {
                                idx = self.get_slots() + idx;
                                if let Some(up_value) = self.get_state().get_up_value(idx) {
                                    res.push(up_value);
                                } else {
                                    res.push(self.get_state_mut().append_up_value(idx))
                                }
                            } else {
                                res.push(self.get_enclosing_up_values().unwrap()[idx].clone());
                            }
                        }

                        res
                    };
                    self.push(Value::new_closure(function, up_values));
                    progress = 2 + count as i32 * 2;
                }
                Call => {
                    let argc = self.read_byte();
                    let idx = self.get_state().stack.len() - argc - 1;
                    let enclosing_closure = self.get_closure().clone();
                    let mut frame = match self.get_state().stack[idx].clone() {
                        Value::Object(Object::Closure(closure)) => {
                            let arity = closure.get_arity();
                            Self::check_arity(arity.clone(), argc).map_err(|value| {
                                let mut err = RuntimeError::new(value);
                                err.push_frame(self);
                                err
                            })?;
                            let n_optionals = argc - arity.required;
                            let ip = if n_optionals == arity.optional {
                                closure.get_start_ip()
                            } else if n_optionals > 0 {
                                closure.get_defaults()[n_optionals]
                            } else {
                                0
                            };
                            Frame::new_closure(
                                self.get_state_mut(),
                                closure,
                                ip,
                                idx,
                                Some(enclosing_closure.get_up_values()),
                            )
                        }
                        Value::Object(Object::Native(native)) => {
                            Frame::new_native(self.get_state_mut(), native, idx)
                        }
                        _ => return Err(self.string_to_err("يمكن فقط استدعاء الدوال".to_string())),
                    };
                    progress = 2;
                    match frame.run(argc) {
                        Ok(returned) => {
                            self.truncate(idx);
                            self.push(returned.unwrap());
                        }
                        Err(mut err) => match self.get_handlers_mut().pop() {
                            Some(Handler { slots, ip }) => {
                                self.truncate(slots);
                                self.push(err.value);
                                progress = (ip - self.get_ip()) as i32;
                            }
                            None => {
                                err.push_frame(self);
                                return Err(err);
                            }
                        },
                    };
                }
                GetUpValue => {
                    let idx = self.read_byte();
                    let up_value = self.get_closure().get_up_value(idx);
                    self.push(match &*up_value.borrow() {
                        UpValue::Open(idx) => self.get(*idx).clone(),
                        UpValue::Closed(up_value) => up_value.clone(),
                    });
                    progress = 2;
                }
                SetUpValue => {
                    let idx = self.read_byte();
                    let up_value = self.get_closure().get_up_value(idx);
                    if up_value.borrow().is_open() {
                        *self.get_mut(up_value.borrow().as_open()) = self.last().clone();
                    } else {
                        *up_value.borrow_mut() = UpValue::Closed(self.last().clone());
                    }
                    progress = 2;
                }
                CloseUpValue => {
                    let idx = self.get_state().stack.len() - 1;
                    self.get_state_mut().close_up_values(idx);
                    self.pop();
                }
                Return => return Ok(Some(self.pop())),
                AppendHandler => {
                    let handler = Handler::new(
                        self.get_state().stack.len(),
                        self.get_ip() + self.read_byte(),
                    );
                    self.get_handlers_mut().push(handler);
                    progress = 3;
                }
                PopHandler => {
                    self.get_handlers_mut().pop();
                }
                Throw => match self.get_handlers_mut().pop() {
                    Some(Handler { slots, ip }) => {
                        let throwed = self.pop();
                        self.truncate(slots);
                        self.push(throwed);
                        progress = (ip - self.get_ip()) as i32;
                    }
                    None => {
                        let mut err = RuntimeError::new(self.pop());
                        err.push_frame(self);
                        return Err(err);
                    }
                },
                Size => {
                    let popped = self.pop();

                    match &popped {
                        Value::Object(Object::String(string)) => {
                            self.push(Value::Number(string.chars().count() as f64));
                        }
                        Value::Object(Object::List(items)) => {
                            self.push(Value::Number(items.borrow().len() as f64));
                        }
                        _ => return Err(self.string_to_err("يجب أن يكون نصاً أو قائمة".to_string())),
                    }
                }
                UnpackList => {
                    let len = self.read_byte();
                    let popped = self.pop();

                    match &popped {
                        Value::Object(Object::List(items)) => {
                            if items.borrow().len() != len {
                                return Err(self.string_to_err("يجب أن يكون عدد العناصر التي ستوزع مساوياً لعدد عناصر الموزع منه".to_string()));
                            }
                            for item in items.borrow().clone() {
                                self.push(item);
                            }
                        }
                        _ => {
                            return Err(self.string_to_err(
                                "يجب أن تكون القيمة التي ستوزع باستخدام '[]' قائمةً".to_string(),
                            ))
                        }
                    }
                    progress = 2;
                }
                UnpackObject => {
                    let stack_len = self.get_state().stack.len();
                    let len = self.read_byte();
                    let keys: Vec<_> = self
                        .get_state_mut()
                        .stack
                        .drain(stack_len - len..)
                        .map(|value| value.as_string())
                        .collect();
                    let items = match self.pop() {
                        Value::Object(Object::Object(items)) => items,
                        _ => {
                            return Err(self.string_to_err(
                                "يجب أن تكون القيمة التي ستوزع باستخدام '{}' كائناً".to_string(),
                            ))
                        }
                    };
                    for key in keys.iter().rev() {
                        if !items.borrow().contains_key(key) {
                            return Err(self.string_to_err(format!("{key} غير موجود في الكائن")));
                        }
                        self.push(items.borrow()[key].clone())
                    }
                    progress = 2;
                }
                PushTmp => {
                    let popped = self.pop();
                    self.get_state_mut().tmps.push(popped)
                }
                FlushTmps => {
                    let mut tmps = vec![];
                    tmps.append(&mut self.get_state_mut().tmps);
                    self.get_state_mut().stack.append(&mut tmps)
                }
                CloneTop => self.push(self.last().clone()),
                Unknown => unreachable!(),
            }
            self.set_ip((self.get_ip() as i32 + progress) as usize);
            if cfg!(feature = "debug-execution") {
                println!("{:#?}", self.get_state().stack);
            }
        }
        Ok(None)
    }
}
