use compiler::chunk::value::{
    self, Arity, ArityType, Closure, DataType, Function, Iterable, Native, Object, Upvalue, Value,
};
use compiler::chunk::{Chunk, Instruction, OpCode::*};
use compiler::error::{Backtrace, RuntimeError};
use parser::token::Token;
use std::collections::{HashMap, LinkedList};
use std::ops::{Deref, DerefMut, Div, Mul, Rem, Sub};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

pub struct Vm {
    tmps: Vec<Value>,
    locals: Vec<Value>,
    globals: HashMap<String, Value>,
    open_upvalues: LinkedList<Rc<RefCell<Upvalue>>>,
}

impl Vm {
    pub fn new() -> Self {
        let qatam_print = Native::new(
            |args: Vec<Value>| {
                println!("{}", args[1]);
                Ok(Value::Nil)
            },
            Arity::new(ArityType::Fixed, 1, 0),
        );

        Self {
            tmps: vec![],
            locals: vec![],
            globals: HashMap::from([("إطبع".to_owned(), Value::from(qatam_print))]),
            open_upvalues: LinkedList::new(),
        }
    }

    fn add_upvalue(&mut self, idx: usize) -> Rc<RefCell<Upvalue>> {
        macro_rules! create_upvalue {
            () => {
                Rc::new(RefCell::new(Upvalue::Open(idx)))
            };
        }

        for (i, upvalue) in self.open_upvalues.clone().into_iter().enumerate() {
            let upvalue_idx = upvalue.borrow().clone().try_into().unwrap();
            match idx {
                x if x < upvalue_idx => {
                    let after = self.open_upvalues.split_off(i);
                    let new_upvalue = create_upvalue!();
                    self.open_upvalues.push_back(Rc::clone(&new_upvalue));
                    for upvalue in after {
                        self.open_upvalues.push_back(upvalue)
                    }
                    return new_upvalue;
                }
                x if x == upvalue_idx => {
                    return upvalue;
                }
                _ => {}
            }
        }
        let new_upvalue = create_upvalue!();
        self.open_upvalues.push_back(Rc::clone(&new_upvalue));
        new_upvalue
    }

    /// Closes the upvalue with `idx` and the ones after it.
    fn close_upvalues(&mut self, idx: usize) {
        loop {
            match self.open_upvalues.back() {
                Some(upvalue) => {
                    let upvalue_idx: usize = upvalue.borrow().clone().try_into().unwrap();
                    if upvalue_idx >= idx {
                        let popped = self.open_upvalues.pop_back().unwrap();
                        *popped.borrow_mut().deref_mut() =
                            Upvalue::Closed(self.locals[upvalue_idx].clone());
                    }
                }
                _ => break,
            }
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), RuntimeError> {
        Frame::new(self, Rc::new(chunk.into())).run()?;
        Ok(())
    }
}

struct Frame<'a> {
    state: &'a mut Vm,
    closure: Rc<Closure>,
    ip: usize,
    slots: usize,
    idx: usize,
    handlers: Vec<Handler>,
}

impl<'a> Frame<'a> {
    fn new(state: &'a mut Vm, closure: Rc<Closure>) -> Self {
        Self {
            state,
            closure,
            ip: 0,
            slots: 0,
            idx: 0,
            handlers: vec![],
        }
    }

    fn new_function(state: &'a mut Vm, closure: Rc<Closure>, argc: usize, idx: usize) -> Self {
        Self {
            ip: closure.start_ip(argc),
            slots: state.locals.len(),
            state,
            closure,
            idx,
            handlers: vec![],
        }
    }

    fn check_type(&self, value: &Value, expected: &[DataType]) -> Result<(), RuntimeError> {
        let received = value.typ();
        if expected.contains(&received) {
            Ok(())
        } else {
            Err(RuntimeError::Type(
                expected.to_owned(),
                received,
                self.token(),
                Backtrace::default(),
            ))
        }
    }

    fn check_arity(&self, arity: &Arity, argc: usize) -> Result<(), RuntimeError> {
        match argc {
            x if x >= arity.required() && x <= arity.required() + arity.optional() => Ok(()),
            x if x > arity.required() + arity.optional() && arity.typ() == ArityType::Variadic => {
                Ok(())
            }
            _ => Err(RuntimeError::InvalidArgc(
                arity.clone(),
                argc,
                self.token(),
                Backtrace::default(),
            )),
        }
    }

    fn chunk(&self) -> &Chunk {
        self.closure.chunk()
    }

    fn token(&self) -> Rc<Token> {
        self.chunk().token(self.ip)
    }

    fn local(&self, idx: usize) -> &Value {
        &self.state.locals[idx]
    }

    fn local_mut(&mut self, idx: usize) -> &mut Value {
        &mut self.state.locals[idx]
    }

    fn push_local(&mut self, value: Value) {
        self.state.locals.push(value)
    }

    fn pop_local(&mut self) -> Value {
        self.state.locals.pop().unwrap()
    }

    fn pop(&mut self) -> Value {
        self.state.tmps.pop().unwrap()
    }

    fn pop_typed(&mut self, expected: &[DataType]) -> Result<Value, RuntimeError> {
        let value = self.pop();
        self.check_type(&value, expected)?;
        Ok(value)
    }

    fn last(&self) -> &Value {
        self.state.tmps.last().unwrap()
    }

    fn last_typed(&self, expected: &[DataType]) -> Result<&Value, RuntimeError> {
        let value = self.last();
        self.check_type(value, expected)?;
        Ok(value)
    }

    fn push(&mut self, value: Value) {
        self.state.tmps.push(value)
    }

    fn run_instr(&mut self, instr: Instruction) -> Result<(Option<Value>, bool), RuntimeError> {
        let mut returned = None;
        let mut advance = true;
        macro_rules! numeric_arith_op {
            ($method:ident) => {{
                let b = self.pop_typed(&[DataType::Number])?;
                let a = self.pop_typed(&[DataType::Number])?;
                self.push(Value::$method(a, b))
            }};
        }
        macro_rules! eq_op {
            ($method:ident) => {{
                let b = self.pop();
                let a = self.pop();
                self.push(Value::from(Value::$method(&a, &b)))
            }};
        }
        macro_rules! numeric_cmp_op {
            ($($ordering:expr),+) => {{
                let b = self.pop_typed(&[DataType::Number])?;
                let a = self.pop_typed(&[DataType::Number])?;
                let res = Value::partial_cmp(&a, &b).unwrap();
                self.push(Value::from($(res == $ordering)||+))
            }};
        }
        macro_rules! jump_if_x_or_pop {
            ($x:expr) => {{
                let offset = instr.read_two_bytes_oper(0);
                if $x {
                    self.ip += offset - instr.size();
                    advance = false;
                } else {
                    self.pop();
                }
            }};
        }
        macro_rules! jump_if_x {
            ($x:expr) => {{
                let offset = instr.read_two_bytes_oper(0);
                if $x {
                    self.ip += offset;
                    advance = false;
                }
            }};
        }
        match instr.op_code() {
            NEG => {
                let value = self.pop_typed(&[DataType::Number])?;
                self.push(-value)
            }
            NOT => {
                let value = self.pop();
                self.push(!value);
            }
            ADD => {
                let b = self.pop();
                let a = self.pop_typed(&[DataType::Number, DataType::String, DataType::List])?;
                self.check_type(&b, &[a.typ()])?;
                self.push(a + b)
            }
            SUB => numeric_arith_op!(sub),
            MUL => numeric_arith_op!(mul),
            DIV => numeric_arith_op!(div),
            REM => numeric_arith_op!(rem),
            EQ => eq_op!(eq),
            NOT_EQ => eq_op!(ne),
            GREATER => numeric_cmp_op!(Ordering::Greater),
            GREATER_EQ => numeric_cmp_op!(Ordering::Greater, Ordering::Equal),
            LESS => numeric_cmp_op!(Ordering::Less),
            LESS_EQ => numeric_cmp_op!(Ordering::Less, Ordering::Equal),
            CONST8 | CONST16 => {
                let idx = instr.read_oper(instr.size() - 1, 0);
                self.push(self.chunk().constant(idx))
            }
            JUMP => {
                let offset = instr.read_two_bytes_oper(0);
                self.ip += offset;
                advance = false;
            }
            JUMP_IF_FALSY_OR_POP => jump_if_x_or_pop!(!self.last().truthy()),
            JUMP_IF_TRUTHY_OR_POP => jump_if_x_or_pop!(self.last().truthy()),
            POP_JUMP_IF_FALSY => jump_if_x!(!self.pop().truthy()),
            POP_JUMP_IF_TRUTHY => jump_if_x!(self.pop().truthy()),
            FOR_ITER => {
                let offset = instr.read_two_bytes_oper(0);
                let iterator: Rc<RefCell<value::Iterator>> = self
                    .last_typed(&[DataType::Iterator])?
                    .clone()
                    .try_into()
                    .unwrap();
                let mut iterator = iterator.borrow_mut();
                match iterator.next() {
                    Some(value) => self.push(value),
                    None => {
                        self.ip += offset;
                        advance = false;
                    }
                }
            }
            LOOP => {
                let offset = instr.read_two_bytes_oper(0);
                self.ip -= offset;
                advance = false;
            }
            GET_LOCAL => {
                let idx = instr.read_byte_oper(0);
                self.push(self.local(self.slots + idx).clone())
            }
            SET_LOCAL => {
                let idx = instr.read_byte_oper(0);
                *self.local_mut(self.slots + idx) = self.last().clone();
            }
            DEF_LOCAL => {
                let value = self.pop();
                self.push_local(value)
            }
            POP_LOCAL => {
                self.pop_local();
            }
            GET_UPVALUE => {
                let idx = instr.read_byte_oper(0);
                self.push(match self.closure.upvalue(idx).borrow().deref() {
                    Upvalue::Closed(value) => value.clone(),
                    Upvalue::Open(idx) => self.local(*idx).clone(),
                })
            }
            SET_UPVALUE => {
                let idx = instr.read_byte_oper(0);
                if let Upvalue::Closed(value) = self.closure.upvalue(idx).borrow_mut().deref_mut() {
                    *value = self.last().clone()
                } else {
                    let idx: usize = self
                        .closure
                        .upvalue(idx)
                        .borrow()
                        .clone()
                        .try_into()
                        .unwrap();
                    *self.local_mut(idx) = self.last().clone();
                }
            }
            CLOSE_UPVALUE => {
                let idx = self.state.locals.len() - 1;
                self.state.close_upvalues(idx);
                self.pop_local();
            }
            GET_GLOBAL8 | GET_GLOBAL16 => {
                let idx = instr.read_oper(instr.size() - 1, 0);
                let name: String = self.chunk().constant(idx).try_into().unwrap();
                let value = match self.state.globals.get(&name) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(RuntimeError::Name(name, self.token(), Backtrace::default()))
                    }
                };
                self.push(value)
            }
            SET_GLOBAL8 | SET_GLOBAL16 => {
                let idx = instr.read_oper(instr.size() - 1, 0);
                let name: String = self.chunk().constant(idx).try_into().unwrap();
                let new_value = self.last().clone();
                match self.state.globals.get_mut(&name) {
                    Some(value) => *value = new_value,
                    None => {
                        return Err(RuntimeError::Name(name, self.token(), Backtrace::default()))
                    }
                }
            }
            DEF_GLOBAL8 | DEF_GLOBAL16 => {
                let idx = instr.read_oper(instr.size() - 1, 0);
                let name: String = self.chunk().constant(idx).try_into().unwrap();
                let value = self.pop();
                if !self.state.globals.contains_key(&name) {
                    self.state.globals.insert(name, value);
                } else {
                    return Err(RuntimeError::AlreadyDefined(
                        name,
                        self.token(),
                        Backtrace::default(),
                    ));
                }
            }
            CLOSURE8 | CLOSURE16 => {
                let idx_size = match instr.op_code() {
                    CLOSURE8 => 1,
                    CLOSURE16 => 2,
                    _ => unreachable!(),
                };
                let idx = instr.read_oper(idx_size, 0);
                let function: Rc<Function> = self.chunk().constant(idx).try_into().unwrap();
                let upvaluec = instr.read_byte_oper(idx_size);
                let mut upvalues = vec![];
                for idx in 0..upvaluec {
                    let offset = idx_size + 1 + idx * 2;
                    let local = instr.read_byte_oper(offset) != 0;
                    let idx = instr.read_byte_oper(offset + 1);
                    if local {
                        upvalues.push(self.state.add_upvalue(idx))
                    } else {
                        upvalues.push(self.closure.upvalue(idx))
                    }
                }
                self.push(Value::from(Closure::new(function, upvalues)))
            }
            CALL => {
                // TODO add stack overflowing
                let argc = instr.read_byte_oper(0);
                let tmps_len = self.state.tmps.len();
                let idx = tmps_len - argc - 1;
                match self.state.tmps[idx].clone() {
                    Value::Object(Object::Closure(closure)) => {
                        self.check_arity(closure.arity(), argc)?;
                        let value = Frame::new_function(self.state, closure, argc, self.idx + 1)
                            .run()?
                            .unwrap();
                        self.push(value)
                    }
                    Value::Object(Object::Native(native)) => {
                        self.check_arity(native.arity(), argc)?;
                        let args = self.state.tmps.drain(idx..).collect::<Vec<_>>();
                        self.push(native.call(args)?)
                    }
                    _ => todo!("Add Uncallable error type"),
                }
            }
            BUILD_VARIADIC => {
                let arity = self.closure.arity();
                let additional = self
                    .state
                    .tmps
                    .drain(arity.required() + arity.optional() + 1..) // https://share.sketchpad.app/22/36b-20f7-cd4981.png
                    .collect::<Vec<_>>();
                self.push(Value::from(additional))
            }
            RET => {
                self.state.close_upvalues(self.slots);
                self.state.locals.drain(self.slots..);
                returned = Some(self.pop())
            }
            BUILD_LIST => {
                let size = instr.read_two_bytes_oper(0);
                let list = self
                    .state
                    .tmps
                    .drain(self.state.tmps.len() - size..)
                    .collect::<Vec<_>>();
                self.push(Value::from(list))
            }
            BUILD_HASH_MAP => {
                let size = instr.read_two_bytes_oper(0);
                let mut hash_map = HashMap::new();
                while hash_map.len() < size {
                    let value = self.pop();
                    let key = self.pop_typed(&[DataType::String])?.try_into().unwrap();
                    hash_map.insert(key, value);
                }
                self.push(Value::from(hash_map))
            }
            GET => {
                let key = self.pop();
                let popped =
                    self.pop_typed(&[DataType::String, DataType::List, DataType::HashMap])?;
                let value = match &popped {
                    Value::String(..) | Value::Object(Object::List(..)) => {
                        let idx: usize = key.try_into().map_err(|_| {
                            RuntimeError::InvalidIdx(self.token(), Backtrace::default())
                        })?;
                        match popped {
                            Value::String(string) => match string.chars().nth(idx) {
                                Some(c) => Value::from(c),
                                None => {
                                    return Err(RuntimeError::OutOfRange(
                                        idx,
                                        string.chars().count(),
                                        self.token(),
                                        Backtrace::default(),
                                    ))
                                }
                            },
                            Value::Object(Object::List(list)) => match list.borrow().get(idx) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(RuntimeError::OutOfRange(
                                        idx,
                                        list.borrow().len(),
                                        self.token(),
                                        Backtrace::default(),
                                    ))
                                }
                            },
                            _ => unreachable!(),
                        }
                    }
                    Value::Object(Object::HashMap(hash_map)) => {
                        self.check_type(&key, &[DataType::String])?;
                        let key: String = key.try_into().unwrap();
                        match hash_map.borrow().get(&key).cloned() {
                            Some(value) => value,
                            None => {
                                return Err(RuntimeError::UndefinedKey(
                                    key,
                                    self.token(),
                                    Backtrace::default(),
                                ))
                            }
                        }
                    }
                    _ => unreachable!(),
                };
                self.push(value)
            }
            SET => {
                let key = self.pop();
                let popped = self.pop_typed(&[DataType::List, DataType::HashMap])?;
                let new_value = self.last().clone();
                match popped {
                    Value::Object(Object::List(list)) => {
                        let idx: usize = key.try_into().map_err(|_| {
                            RuntimeError::InvalidIdx(self.token(), Backtrace::default())
                        })?;
                        match list.borrow_mut().get_mut(idx) {
                            Some(value) => {
                                *value = new_value;
                            }
                            None => {
                                return Err(RuntimeError::OutOfRange(
                                    idx,
                                    list.borrow().len(),
                                    self.token(),
                                    Backtrace::default(),
                                ));
                            }
                        }
                    }
                    Value::Object(Object::HashMap(hash_map)) => {
                        self.check_type(&key, &[DataType::String])?;
                        let key: String = key.try_into().unwrap();
                        hash_map.borrow_mut().insert(key, new_value);
                    }
                    _ => unreachable!(),
                }
            }
            APPEND_HANDLER => {
                let offset = instr.read_two_bytes_oper(0);
                self.handlers
                    .push(Handler::new(self.ip + offset, self.state.locals.len()))
            }
            POP_HANDLER => {
                self.handlers.pop();
            }
            THROW => {
                let value = self.pop();
                return Err(RuntimeError::User(
                    value,
                    self.token(),
                    Backtrace::default(),
                ));
            }
            ITER => {
                let iterable: Iterable = self
                    .last_typed(&[DataType::String, DataType::List])?
                    .clone()
                    .try_into()
                    .unwrap();
                self.push(Value::from(iterable))
            }
            UNPACK_LIST => {
                let to = instr.read_two_bytes_oper(0);
                let popped = self.pop_typed(&[DataType::List])?;
                let list: Rc<RefCell<Vec<Value>>> = popped.try_into().unwrap();
                let list = list.borrow();
                if list.len() != to {
                    return Err(RuntimeError::ListUnpack(
                        to,
                        list.len(),
                        self.token(),
                        Backtrace::default(),
                    ));
                }
                for value in list.iter() {
                    self.push(value.clone())
                }
            }
            UNPACK_HASH_MAP => {
                let propc = instr.read_two_bytes_oper(0);
                let keys = {
                    let mut tmp = vec![];
                    for idx in (0..propc).rev() {
                        if instr.read_byte_oper(2 + idx) != 0 {
                            let default = self.pop();
                            let key: String = self.pop().try_into().unwrap();
                            tmp.push((key, Some(default)))
                        } else {
                            let key: String = self.pop().try_into().unwrap();
                            tmp.push((key, None))
                        };
                    }
                    tmp
                };
                let popped = self.pop_typed(&[DataType::HashMap])?;
                let hash_map: Rc<RefCell<HashMap<String, Value>>> = popped.try_into().unwrap();
                let hash_map = hash_map.borrow();
                for (key, default) in keys {
                    let value = match hash_map.get(&key).cloned() {
                        Some(value) => value,
                        None => match default {
                            Some(default) => default,
                            None => {
                                return Err(RuntimeError::UndefinedKey(
                                    key,
                                    self.token(),
                                    Backtrace::default(),
                                ))
                            }
                        },
                    };
                    self.push(value)
                }
            }
            POP => {
                self.pop();
            }
            DUP => {
                let value = self.last().clone();
                self.push(value)
            }
            UNKNOWN => unreachable!(),
        }
        Ok((returned, advance))
    }

    fn run(&mut self) -> Result<Option<Value>, RuntimeError> {
        while let Some(instr) = self.closure.chunk().read(self.ip) {
            if cfg!(feature = "verbose") {
                println!("{} => {:?}", self.ip, self.state.tmps)
            }
            let size = instr.size();
            match self.run_instr(instr) {
                Ok((returned, advance)) => {
                    match returned {
                        Some(returned) => return Ok(Some(returned)),
                        None => {}
                    }
                    if advance {
                        self.ip += size;
                    }
                }
                Err(mut err) => match self.handlers.pop() {
                    Some(handler) => {
                        self.state.close_upvalues(handler.slots());
                        self.state.locals.drain(handler.slots()..);
                        self.push(err.into());
                        self.ip = handler.ip();
                    }
                    None => {
                        err.backtrace_mut().push(self.closure.name(), self.token());
                        return Err(err);
                    }
                },
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct Handler {
    ip: usize,
    slots: usize,
}

impl Handler {
    fn new(ip: usize, slots: usize) -> Self {
        Self { ip, slots }
    }

    fn ip(&self) -> usize {
        self.ip
    }

    fn slots(&self) -> usize {
        self.slots
    }
}
