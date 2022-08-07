pub mod value;

use lexer::token::Token;
use std::{cell::RefCell, fmt, rc::Rc};
use value::{Function, Value};

fn combine(a: u8, b: u8) -> u16 {
    (b as u16) << 8 | (a as u16)
}

fn split(bytes: u16) -> (u8, u8) {
    (bytes as u8, (bytes >> 8) as u8)
}

macro_rules! byte_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::From<u8> for $name {
            fn from(v: u8) -> Self {
                match v {
                    $(x if x == $name::$vname as u8 => $name::$vname,)*
                    _ => $name::UNKNOWN,
                }
            }
        }
    }
}

byte_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Instruction {
        /// Implements `Value::neg` on TOS.
        NEGATE,
        /// Implements `Value::not` on TOS.
        NOT,
        /// Implmenets `Value::add` on TOS1 and TOS.
        ADD,
        /// Implements `Value::sub` on TOS1 and TOS.
        SUBTRACT,
        /// Implements `Value::mul` on TOS1 and TOS.
        MULTIPLY,
        /// Implements `Value::div` on TOS1 and TOS.
        DIVIDE,
        /// Implements `Value::rem` on TOS1 and TOS.
        REMAINDER,
        /// Uses `Value::eq`.
        EQUAL,
        NOT_EQUAL,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Greater`.
        GREATER,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Greater` or `Ordering::Equal`.
        GREATER_EQUAL,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Less`.
        LESS,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Less` or `Ordering::Equal`.
        LESS_EQUAL,
        /// `CONSTANT8 <idx: u8>`
        ///
        /// Pushes `constants[idx]` into the stack.
        CONSTANT8,
        /// `CONSTANT16 <idx: u16>`
        ///
        /// Pushes `constants[idx]` into the stack.
        CONSTANT16,
        /// `JUMP <offset: u16>`
        ///
        /// Adds `offset` to the ip.
        JUMP,
        /// `JUMP_IF_FALSE_OR_POP <offset: u16>`
        ///
        /// Jumps if TOS is false otherwise TOS is popped.
        JUMP_IF_FALSE_OR_POP,
        /// `JUMP_IF_TRUE_OR_POP <offset: u16>`
        ///
        /// Jumps if TOS is true otherwise TOS is popped.
        JUMP_IF_TRUE_OR_POP,
        /// `POP_JUMP_IF_FALSE <offset: u16>`
        ///
        /// Jumps if TOS is false, TOS is popped.
        POP_JUMP_IF_FALSE,
        /// `POP_JUMP_IF_TRUE <offset: u16>`
        ///
        /// Jumps if TOS is true, TOS is popped.
        POP_JUMP_IF_TRUE,
        /// `LOOP <offset: u16>`
        ///
        /// Subtracts `offset` from the ip.
        LOOP,
        /// `GET_LOCAL <idx: u8>`
        ///
        /// Pushes `locals[idx]` into the stack.
        GET_LOCAL,
        /// `SET_LOCAL <idx: u8>`
        ///
        /// Sets `locals[idx]` to TOS, TOS stays on the stack.
        SET_LOCAL,
        /// Puts TOS in the variables stack, TOS is popped.
        DEFINE_LOCAL,
        /// `GET_UPVALUE <idx: u8>`
        ///
        /// Pushes `frame.closure.upvalues[idx]` into the stack.
        GET_UPVALUE,
        /// `SET_UPVALUE <idx: u8>`
        ///
        /// Sets `frame.closure.upvalues[idx]` to TOS, TOS stays on the stack.
        SET_UPVALUE,
        /// Closes the upvalues tied to TOS, TOS is popped.
        CLOSE_UPVALUE,
        /// `GET_GLOBAL8 <idx: u8>`
        ///
        /// Pushes `globals[constants[idx]]` into the stack.
        GET_GLOBAL8,
        /// `GET_GLOBAL16 <idx: u16>`
        ///
        /// Pushes `globals[constants[idx]]` into the stack.
        GET_GLOBAL16,
        /// `SET_GLOBAL8 <idx: u8>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS stays on the stack.
        SET_GLOBAL8,
        /// `SET_GLOBAL16 <idx: u16>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS stays on the stack.
        SET_GLOBAL16,
        /// `DEFINE_GLOBAL8 <idx: u8>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS is popped.
        DEFINE_GLOBAL8,
        /// `DEFINE_GLOBAL16 <idx: u16>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS is popped.
        DEFINE_GLOBAL16,
        /// `CLOSURE8 <idx: u8> <upvaluec: u8> <(local: bool, idx: u8)>...`
        ///
        /// Expects TOS to be a `Function`, TOS gets replaced with the result.
        ///
        /// Builds a closure with TOS and the operands.
        CLOSURE8,
        /// `CLOSURE16 <idx: u16> <upvaluec: u8> <(local: bool, idx: u8)>...`
        ///
        /// Expects TOS to be a `Function`, TOS gets replaced with the result.
        ///
        /// Builds a closure with TOS and the operands.
        CLOSURE16,
        /// `CALL <argc: u8>`
        ///
        /// Expects stack length - `argc` - 1 to be a callable, i.e, `Function` or `Native`.
        ///
        /// Pushes a new call stack frame into the stack.
        CALL,
        /// Leaves the required and optional params and reduces the rest into a list.
        BUILD_VARIADIC,
        /// Pops the current call stack frame, its arguments and locals, and pushes TOS.
        RETURN,
        /// `BUILD_LIST <size: u16>`
        ///
        /// Pops `size` from the stack and pushes them back as a list.
        BUILD_LIST,
        /// `BUILD_OBJECT <size: u16>`
        ///
        /// Expects key-value pairs to be on the stack.
        BUILD_OBJECT,
        /// `GET8 <idx: u8>`
        ///
        /// Pushes `TOS[constants[idx]]` into the stack, TOS is popped.
        GET8,
        /// `GET16 <idx: u16>`
        ///
        /// Pushes `TOS[constants[idx]]` into the stack, TOS is popped.
        GET16,
        /// `SET8 <idx: u8>`
        ///
        /// Sets `TOS1[constants[idx]]` to TOS, TOS1 is popped.
        SET8,
        /// `SET16 <idx: u16>`
        ///
        /// Sets `TOS[constants[idx]]` to TOS, TOS1 is popped.
        SET16,
        /// `APPEND_HANDLER <offset: u16>`
        ///
        /// Appends an errors handler which contains the start of the block's slots and ip of the catch block start.
        APPEND_HANDLER,
        /// Pops the last handler.
        POP_HANDLER,
        /// Throws TOS.
        THROW,
        /// Turns TOS into an iterator.
        ///
        /// Expects TOS to be a string or list.
        ITER,
        /// `UNPACK_LIST <to: u16>`
        ///
        /// Spreads the list into the stack.
        ///
        /// If `to` isn't equal to the length of TOS.
        ///
        /// Expects TOS to be a list.
        UNPACK_LIST,
        /// `UNPACK_HASH_MAP <propc: u16> <default: bool>...`
        ///
        /// Expects the keys and default values to be on the stack.
        ///
        /// Expects TOS to be a a hash map.
        UNPACK_HASH_MAP,
        /// Pops TOS off the stack.
        POP,
        /// Duplicates TOS.
        CLONE_TOP,
        UNKNOWN,
    }
}

use Instruction::*;

const NIL_CONST: usize = 0;
const TRUE_CONST: usize = 1;
const FALSE_CONST: usize = 2;

#[derive(Clone)]
pub struct Chunk {
    bytes: RefCell<Vec<u8>>,
    constants: RefCell<Vec<Value>>,
    tokens: RefCell<Vec<Option<Rc<Token>>>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bytes: RefCell::new(vec![]),
            constants: RefCell::new(vec![Value::Nil, Value::Bool(true), Value::Bool(false)]),
            tokens: RefCell::new(vec![]),
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.borrow().len()
    }

    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.bytes.borrow().get(offset).cloned()
    }

    pub fn constant(&self, idx: usize) -> Value {
        self.constants.borrow().get(idx).unwrap().clone()
    }

    pub fn token(&self, ip: usize) -> Rc<Token> {
        Rc::clone(&self.tokens.borrow()[ip].as_ref().unwrap())
    }

    fn write_instr(&self, instr: Instruction, token: Rc<Token>) {
        self.bytes.borrow_mut().push(instr as u8);
        self.tokens.borrow_mut().push(Some(token));
    }

    fn write_byte(&self, byte: usize) -> Result<(), ()> {
        if byte <= u8::MAX.into() {
            self.bytes.borrow_mut().push(byte as u8);
            self.tokens.borrow_mut().push(None);
            Ok(())
        } else {
            Err(())
        }
    }

    fn write_two_bytes(&self, two_bytes: usize) -> Result<(), ()> {
        if two_bytes <= u16::MAX.into() {
            let (byte1, byte2) = split(two_bytes as u16);
            self.write_byte(byte1 as usize).ok();
            self.write_byte(byte2 as usize).ok();
            Ok(())
        } else {
            Err(())
        }
    }

    fn rewrite_two_bytes(&self, idx: usize, value: usize) -> Result<(), ()> {
        if value <= u16::MAX.into() {
            let (byte1, byte2) = split(value as u16);
            self.bytes.borrow_mut()[idx] = byte1;
            self.bytes.borrow_mut()[idx + 1] = byte2;
            Ok(())
        } else {
            Err(())
        }
    }

    fn add_constant(&self, value: Value) -> usize {
        match &value {
            Value::Nil => return NIL_CONST,
            Value::Bool(val) => return if *val { TRUE_CONST } else { FALSE_CONST },
            Value::String(string) => {
                for (idx, const_) in self.constants.borrow().iter().enumerate() {
                    if let Value::String(string_2) = const_ {
                        if string_2 == string {
                            return idx;
                        }
                    }
                }
            }
            _ => {}
        }
        let idx = self.constants.borrow().len();
        self.constants.borrow_mut().push(value);
        idx
    }

    /// `instr` must be `NEGATE`, `NOT`, `ADD`, `SUBTRACT`, `MULTIPLY`, `DIVIDE`, `REMAINDER`, `EQUAL`, `GREATER`, `GREATER_EQUAL`, `LESS`, `LESS_EQUAL`, `DEFINE_LOCAL`, `CLOSE_UPVALUE`, `BUILD_VARIADIC`, `RETURN`, `POP_HANDLER`, `THROW`, `ITER`, `POP`, or `CLONE_TOP`.
    pub fn write_instr_no_operands(&self, instr: Instruction, token: Rc<Token>) {
        self.write_instr(instr, token)
    }

    /// `instr` must be `GET_LOCAL`, `SET_LOCAL`, `GET_UPVALUE`, or `SET_UPVALUE`.
    ///
    /// `token` must be of type `Identifier`.
    ///
    /// Fails when `idx` is greater than 255.
    pub fn write_instr_idx(
        &self,
        instr: Instruction,
        token: Rc<Token>,
        idx: usize,
    ) -> Result<(), ()> {
        self.write_instr(instr, token);
        self.write_byte(idx)
    }

    /// `instr` must be (`CONSTANT8`, `CONSTANT16`), (`GET_GLOBAL8`, `GET_GLOBAL16`), (`SET_GLOBAL8`, `SET_GLOBAL16`), (`DEFINE_GLOBAL8`, `DEFINE_GLOBAL16`), (`GET8`, `GET_16`), or (`SET8`, `SET16`).
    ///
    /// Fails when the chunk already has 65536 constants.
    pub fn write_instr_const(
        &self,
        (u8_instr, u16_instr): (Instruction, Instruction),
        token: Rc<Token>,
        value: Value,
    ) -> Result<(), ()> {
        match self.add_constant(value) {
            idx if idx <= u8::MAX.into() => {
                self.write_instr(u8_instr, token);
                self.write_byte(idx).ok();
                Ok(())
            }
            idx if idx <= u16::MAX.into() => {
                self.write_instr(u16_instr, token);
                self.write_two_bytes(idx).ok();
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// `instr` must be `JUMP`, `POP_JUMP_IF_FALSE`, `POP_JUMP_IF_TRUE`, `JUMP_IF_FALSE_OR_POP`, `JUMP_IF_TRUE_OR_POP`, or `APPEND_HANDLER`.
    ///
    /// `token` must be of type `If`, `ElseIf`, `Else`, `While`, `Loop`, `For`, `And`, or `Or`.
    ///
    /// Returns a closure that ends the jump when called.
    pub fn write_jump(
        &self,
        instr: Instruction,
        token: Rc<Token>,
    ) -> Box<dyn FnOnce() -> Result<(), ()> + '_> {
        let idx = self.len();
        self.write_instr(instr, token);
        self.write_two_bytes(0).ok();
        Box::new(move || {
            let offset = self.len() - idx;
            if offset <= u16::MAX.into() {
                self.rewrite_two_bytes(idx + 1, offset)?;
                Ok(())
            } else {
                Err(())
            }
        })
    }

    /// `token` must be `While`, `Loop` or `For`.
    ///
    /// Fails when chunk length - `ip` is greater than 65535.
    pub fn write_loop(&self, token: Rc<Token>, ip: usize) -> Result<(), ()> {
        let offset = self.len() - ip;
        self.write_instr(LOOP, token);
        self.write_two_bytes(offset)
    }

    /// `token` must be of type `Function` or `Pipe`.
    ///
    /// Can fail while appending `function` or because `upvalues` length is greater than 255.
    pub fn write_closure(
        &self,
        token: Rc<Token>,
        function: Function,
        upvalues: Vec<(bool, usize)>,
    ) -> Result<(), ()> {
        self.write_instr_const((CLOSURE8, CLOSURE16), token, Value::from(function))?;
        self.write_byte(upvalues.len())?;
        for (local, idx) in upvalues {
            self.write_byte(if local { 1 } else { 0 }).ok();
            self.write_byte(idx).ok();
        }
        Ok(())
    }

    /// `token` must be the one of the callee.
    ///
    /// Fails when `argc` is greater than 255.
    pub fn write_call(&self, token: Rc<Token>, argc: usize) -> Result<(), ()> {
        self.write_instr(CALL, token);
        self.write_byte(argc)
    }

    /// `token` must be the of type `OBrace` or `OBracket`.
    ///
    /// `instr` must be `BUILD_LIST` or `BUILD_OBJECT`.
    ///
    /// Fails when `size` is greater than 65535.
    pub fn write_build(&self, instr: Instruction, token: Rc<Token>, size: usize) -> Result<(), ()> {
        self.write_instr(instr, token);
        self.write_two_bytes(size)
    }

    /// `token` must be of type `OBrace`.
    ///
    /// Expects that all of the keys along with their default values have been written before in the form `key default?`.
    ///
    /// `defaults` is an array of flags that reflects the structure of the keys and default values already written.
    ///
    /// Fails when `defaults` length is greater than 65535.
    pub fn write_hash_map_unpack(&self, token: Rc<Token>, defaults: Vec<bool>) -> Result<(), ()> {
        self.write_instr(UNPACK_HASH_MAP, token);
        self.write_two_bytes(defaults.len())?;
        for flag in defaults {
            self.write_byte(if flag { 1 } else { 0 })?;
        }
        Ok(())
    }

    /// `token` msut be of type `OBracket`.
    ///
    /// Fails when `to` is greater than 65535
    pub fn write_list_unpack(&self, token: Rc<Token>, to: usize) -> Result<(), ()> {
        self.write_instr(UNPACK_LIST, token);
        self.write_two_bytes(to)
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ip = 0;
        let mut cur_line = 0;

        macro_rules! byte_oper {
            ($($offset:expr)?) => {
                self.bytes.borrow()[ip + 1$( + ($offset))?] as usize
            };
        }
        macro_rules! two_bytes_oper {
            ($($offset:expr)?) => {
                combine(self.bytes.borrow()[ip + 1$( + ($offset))?], self.bytes.borrow()[ip + 2$( + ($offset))?]) as usize
            };
        }

        while ip < self.len() {
            let instr = self.bytes.borrow()[ip].into();
            let (line, _) = self.token(ip).pos();
            if line != cur_line {
                write!(f, "{line:^5?} | ")?;
                cur_line = line;
            } else {
                write!(f, "{} | ", " ".repeat(5))?;
            }
            write!(f, "{ip:<05} {:20}", format!("{instr:?}"))?;
            match instr {
                NEGATE | NOT | ADD | SUBTRACT | MULTIPLY | DIVIDE | REMAINDER | EQUAL
                | NOT_EQUAL | GREATER | GREATER_EQUAL | LESS | LESS_EQUAL | DEFINE_LOCAL
                | CLOSE_UPVALUE | BUILD_VARIADIC | RETURN | POP_HANDLER | THROW | ITER | POP
                | CLONE_TOP => {
                    write!(f, "\n")?;
                    ip += 1;
                }
                GET_LOCAL | SET_LOCAL | GET_UPVALUE | SET_UPVALUE => {
                    let idx = byte_oper!();
                    writeln!(f, " {idx}")?;
                }
                CONSTANT8 | GET_GLOBAL8 | SET_GLOBAL8 | DEFINE_GLOBAL8 | GET8 | SET8 => {
                    let idx = byte_oper!();
                    writeln!(f, " {idx} ({})", self.constant(idx))?;
                    ip += 2;
                }
                CONSTANT16 | GET_GLOBAL16 | SET_GLOBAL16 | DEFINE_GLOBAL16 | GET16 | SET16 => {
                    let idx = two_bytes_oper!();
                    writeln!(f, " {idx} ({})", self.constant(idx))?;
                    ip += 3;
                }
                JUMP | POP_JUMP_IF_FALSE | POP_JUMP_IF_TRUE | JUMP_IF_FALSE_OR_POP
                | JUMP_IF_TRUE_OR_POP | LOOP | APPEND_HANDLER => {
                    let offset = two_bytes_oper!();
                    writeln!(f, " {offset} (to {})", ip + offset)?;
                    ip += 3;
                }
                CLOSURE8 | CLOSURE16 => {
                    let upvaluec = byte_oper!(if instr == CLOSURE8 { 1 } else { 2 });
                    for idx in 0..upvaluec {
                        let local = byte_oper!(idx * 2 + 1) != 0;
                        let idx = byte_oper!(idx * 2 + 2);
                        write!(f, "{:?}", (local, idx))?;
                    }
                    ip += 2 + upvaluec * 2;
                }
                CALL => {
                    let argc = byte_oper!();
                    writeln!(f, " {argc}")?;
                    ip += 2;
                }
                BUILD_LIST | BUILD_OBJECT => {
                    let size = two_bytes_oper!();
                    writeln!(f, " {size}")?;
                    ip += 3;
                }
                UNPACK_HASH_MAP => {
                    let propc = two_bytes_oper!();
                    for idx in 0..propc {
                        let default = byte_oper!(idx + 2) != 0;
                        write!(f, " {default}")?;
                    }
                    write!(f, "\n")?;
                    ip += 3 + propc;
                }
                UNPACK_LIST => {
                    let to = two_bytes_oper!();
                    writeln!(f, " {to}")?;
                    ip += 3;
                }
                UNKNOWN => unreachable!(),
            }
        }
        Ok(())
    }
}
