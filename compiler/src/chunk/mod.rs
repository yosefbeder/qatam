pub mod value;

use parser::token::Token;
use std::{fmt, rc::Rc};
use value::{Function, Object, Value};

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
        NEG,
        /// Implements `Value::not` on TOS.
        NOT,
        /// Implmenets `Value::add` on TOS1 and TOS.
        ADD,
        /// Implements `Value::sub` on TOS1 and TOS.
        SUB,
        /// Implements `Value::mul` on TOS1 and TOS.
        MUL,
        /// Implements `Value::div` on TOS1 and TOS.
        DIV,
        /// Implements `Value::rem` on TOS1 and TOS.
        REM,
        /// Uses `Value::eq`.
        EQ,
        NOT_EQ,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Greater`.
        GREATER,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Greater` or `Ordering::Equal`.
        GREATER_EQ,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Less`.
        LESS,
        /// Implemnts `Value::partial_cmp` on TOS1 and TOS, accepting `Ordering::Less` or `Ordering::Equal`.
        LESS_EQ,
        /// `CONST8 <idx: u8>`
        ///
        /// Pushes `constants[idx]` into the stack.
        CONST8,
        /// `CONST16 <idx: u16>`
        ///
        /// Pushes `constants[idx]` into the stack.
        CONST16,
        /// `JUMP <offset: u16>`
        ///
        /// Adds `offset` to the ip.
        JUMP,
        /// `JUMP_IF_FALSY_OR_POP <offset: u16>`
        ///
        /// Jumps if TOS is false otherwise TOS is popped.
        JUMP_IF_FALSY_OR_POP,
        /// `JUMP_IF_TRUTHY_OR_POP <offset: u16>`
        ///
        /// Jumps if TOS is true otherwise TOS is popped.
        JUMP_IF_TRUTHY_OR_POP,
        /// `POP_JUMP_IF_FALSY <offset: u16>`
        ///
        /// Jumps if TOS is false, TOS is popped.
        POP_JUMP_IF_FALSY,
        /// `POP_JUMP_IF_TRUTHY <offset: u16>`
        ///
        /// Jumps if TOS is true, TOS is popped.
        POP_JUMP_IF_TRUTHY,
        /// `FOR_ITER <offset: u16>`
        ///
        /// Keeps advancing the iterator (TOS) pushing the result into the stack until `None` is returned.
        ///
        /// When `None` is returned by the iterator it pops TOS and jumps.
        FOR_ITER,
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
        DEF_LOCAL,
        /// Pops TOVS.
        POP_LOCAL,
        /// `GET_UPVALUE <idx: u8>`
        ///
        /// Pushes `frame.closure.upvalues[idx]` into the stack.
        GET_UPVALUE,
        /// `SET_UPVALUE <idx: u8>`
        ///
        /// Sets `frame.closure.upvalues[idx]` to TOS, TOS stays on the stack.
        SET_UPVALUE,
        /// Closes the upvalues tied to TOVS, TOVS is popped.
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
        /// `DEF_GLOBAL8 <idx: u8>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS is popped.
        DEF_GLOBAL8,
        /// `DEF_GLOBAL16 <idx: u16>`
        ///
        /// Sets `globals[constants[idx]]` to TOS, TOS is popped.
        DEF_GLOBAL16,
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
        RET,
        /// `BUILD_LIST <size: u16>`
        ///
        /// Pops `size` from the stack and pushes them back as a list.
        BUILD_LIST,
        /// `BUILD_HASH_MAP <size: u16>`
        ///
        /// Expects key-value pairs to be on the stack.
        BUILD_HASH_MAP,
        /// `GET`
        ///
        /// Implements `TOS1[TOS]`, TOS and TOS1 are popped.
        GET,
        /// `SET`
        ///
        /// Implements `TOS1[TOS] = TOS2`, TOS and TOS1 are popped.
        SET,
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
        /// Pops TOS.
        POP,
        /// Duplicates TOS.
        DUP,
        UNKNOWN,
    }
}

use Instruction::*;

const NIL_CONST: usize = 0;
const TRUE_CONST: usize = 1;
const FALSE_CONST: usize = 2;

#[derive(Clone)]
pub struct Chunk {
    bytes: Vec<u8>,
    constants: Vec<Value>,
    tokens: Vec<Option<Rc<Token>>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            constants: vec![Value::Nil, Value::Bool(true), Value::Bool(false)],
            tokens: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(offset).cloned()
    }

    pub fn constant(&self, idx: usize) -> Value {
        self.constants.get(idx).unwrap().clone()
    }

    pub fn token(&self, ip: usize) -> Rc<Token> {
        Rc::clone(&self.tokens[ip].as_ref().unwrap())
    }

    fn write_instr(&mut self, instr: Instruction, token: Rc<Token>) {
        self.bytes.push(instr as u8);
        self.tokens.push(Some(token));
    }

    fn write_byte(&mut self, byte: usize) -> Result<(), ()> {
        if byte <= u8::MAX.into() {
            self.bytes.push(byte as u8);
            self.tokens.push(None);
            Ok(())
        } else {
            Err(())
        }
    }

    fn write_two_bytes(&mut self, two_bytes: usize) -> Result<(), ()> {
        if two_bytes <= u16::MAX.into() {
            let (byte1, byte2) = split(two_bytes as u16);
            self.write_byte(byte1 as usize).ok();
            self.write_byte(byte2 as usize).ok();
            Ok(())
        } else {
            Err(())
        }
    }

    fn rewrite_two_bytes(&mut self, idx: usize, value: usize) -> Result<(), ()> {
        if value <= u16::MAX.into() {
            let (byte1, byte2) = split(value as u16);
            self.bytes[idx] = byte1;
            self.bytes[idx + 1] = byte2;
            Ok(())
        } else {
            Err(())
        }
    }

    fn add_constant(&mut self, value: Value) -> usize {
        match &value {
            Value::Nil => return NIL_CONST,
            Value::Bool(val) => return if *val { TRUE_CONST } else { FALSE_CONST },
            Value::String(string) => {
                for (idx, const_) in self.constants.iter().enumerate() {
                    if let Value::String(string_2) = const_ {
                        if string_2 == string {
                            return idx;
                        }
                    }
                }
            }
            _ => {}
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }

    /// `instr` must be `NEG`, `NOT`, `ADD`, `SUB`, `MUL`, `DIV`, `REM`, `EQ`, `GREATER`, `GREATER_EQ`, `LESS`, `LESS_EQ`, `DEF_LOCAL`, `GET`, `SET`, `CLOSE_UPVALUE`, `BUILD_VARIADIC`, `RET`, `POP_HANDLER`, `THROW`, `ITER`, `POP`, or `DUP`.
    pub fn write_instr_no_operands(&mut self, instr: Instruction, token: Rc<Token>) {
        self.write_instr(instr, token)
    }

    /// `instr` must be `GET_LOCAL`, `SET_LOCAL`, `GET_UPVALUE`, or `SET_UPVALUE`.
    ///
    /// Fails when `idx` is greater than 255.
    pub fn write_instr_idx(
        &mut self,
        instr: Instruction,
        token: Rc<Token>,
        idx: usize,
    ) -> Result<(), ()> {
        self.write_instr(instr, token);
        self.write_byte(idx)
    }

    /// `instr` must be (`CONST8`, `CONST16`), (`GET_GLOBAL8`, `GET_GLOBAL16`), (`SET_GLOBAL8`, `SET_GLOBAL16`), (`DEF_GLOBAL8`, `DEF_GLOBAL16`), (`GET8`, `GET_16`), or (`SET8`, `SET16`).
    ///
    /// Fails when the chunk already has 65536 constants.
    pub fn write_instr_const(
        &mut self,
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

    /// `instr` must be `JUMP`, `POP_JUMP_IF_FALSE`, `POP_JUMP_IF_TRUE`, `JUMP_IF_FALSE_OR_POP`, `JUMP_IF_TRUE_OR_POP`, `FOR_ITER`, or `APPEND_HANDLER`.
    ///
    /// Returns its indx
    pub fn write_jump(&mut self, instr: Instruction, token: Rc<Token>) -> usize {
        let idx = self.len();
        self.write_instr(instr, token);
        self.write_two_bytes(0).ok();
        idx
    }

    /// Makes the jump at `ip` point to the current ip.
    pub fn settle_jump(&mut self, ip: usize) -> Result<(), ()> {
        let offset = self.len() - ip;
        if offset <= u16::MAX.into() {
            self.rewrite_two_bytes(ip + 1, offset)
        } else {
            Err(())
        }
    }

    /// Fails when chunk length - `ip` is greater than 65535.
    pub fn write_loop(&mut self, token: Rc<Token>, ip: usize) -> Result<(), ()> {
        let offset = self.len() - ip;
        self.write_instr(LOOP, token);
        self.write_two_bytes(offset)
    }

    /// Can fail while appending `function` or because `upvalues` length is greater than 255.
    pub fn write_closure(
        &mut self,
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

    /// Fails when `argc` is greater than 255.
    pub fn write_call(&mut self, token: Rc<Token>, argc: usize) -> Result<(), ()> {
        self.write_instr(CALL, token);
        self.write_byte(argc)
    }

    /// `instr` must be `BUILD_LIST` or `BUILD_HASH_MAP`.
    ///
    /// Fails when `size` is greater than 65535.
    pub fn write_build(
        &mut self,
        instr: Instruction,
        token: Rc<Token>,
        size: usize,
    ) -> Result<(), ()> {
        self.write_instr(instr, token);
        self.write_two_bytes(size)
    }

    /// Expects that all of the keys along with their default values have been written before in the form `key default?`.
    ///
    /// `defaults` is an array of flags that reflects the structure of the keys and default values already written.
    ///
    /// Fails when `defaults` length is greater than 65535.
    pub fn write_hash_map_unpack(
        &mut self,
        token: Rc<Token>,
        defaults: Vec<bool>,
    ) -> Result<(), ()> {
        self.write_instr(UNPACK_HASH_MAP, token);
        self.write_two_bytes(defaults.len())?;
        for flag in defaults {
            self.write_byte(if flag { 1 } else { 0 })?;
        }
        Ok(())
    }

    /// Fails when `to` is greater than 65535
    pub fn write_list_unpack(&mut self, token: Rc<Token>, to: usize) -> Result<(), ()> {
        self.write_instr(UNPACK_LIST, token);
        self.write_two_bytes(to)
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const LINE_WIDTH: usize = 5;
        const OFFSET_WIDTH: usize = 5;
        const INSTRUCTION_WIDTH: usize = 20;
        let mut ip = 0;
        let mut cur_line = 0;
        let mut inners = vec![];

        macro_rules! byte_oper {
            ($($offset:expr)?) => {
                self.bytes[ip + 1$( + ($offset))?] as usize
            };
        }
        macro_rules! two_bytes_oper {
            ($($offset:expr)?) => {
                combine(self.bytes[ip + 1$( + ($offset))?], self.bytes[ip + 2$( + ($offset))?]) as usize
            };
        }

        while ip < self.len() {
            let instr = self.bytes[ip].into();
            let token = self.token(ip);
            let (line, _) = token.pos();
            if line != cur_line {
                if cur_line != 0 {
                    write!(f, "\n")?
                }
                write!(f, "{line:^LINE_WIDTH$?} ")?;
                cur_line = line;
            } else {
                write!(f, "{} ", " ".repeat(LINE_WIDTH))?;
            }
            write!(
                f,
                "{ip:>OFFSET_WIDTH$} {:INSTRUCTION_WIDTH$}",
                format!("{instr:?}")
            )?;
            match instr {
                NEG | NOT | ADD | SUB | MUL | DIV | REM | EQ | NOT_EQ | GREATER | GREATER_EQ
                | LESS | LESS_EQ | POP_LOCAL | CLOSE_UPVALUE | BUILD_VARIADIC | RET
                | POP_HANDLER | THROW | ITER | POP | DUP | GET | SET => {
                    write!(f, "\n")?;
                    ip += 1;
                }
                DEF_LOCAL => {
                    writeln!(f, " ({})", token.lexeme())?;
                    ip += 1;
                }
                GET_LOCAL | SET_LOCAL | GET_UPVALUE | SET_UPVALUE => {
                    let idx = byte_oper!();
                    writeln!(f, " {idx} ({})", token.lexeme())?;
                    ip += 2;
                }
                CONST8 | GET_GLOBAL8 | SET_GLOBAL8 | DEF_GLOBAL8 => {
                    let idx = byte_oper!();
                    writeln!(f, " {idx} ({})", self.constant(idx))?;
                    ip += 2;
                }
                CONST16 | GET_GLOBAL16 | SET_GLOBAL16 | DEF_GLOBAL16 => {
                    let idx = two_bytes_oper!();
                    writeln!(f, " {idx} ({})", self.constant(idx))?;
                    ip += 3;
                }
                JUMP
                | POP_JUMP_IF_FALSY
                | POP_JUMP_IF_TRUTHY
                | JUMP_IF_FALSY_OR_POP
                | JUMP_IF_TRUTHY_OR_POP
                | FOR_ITER
                | APPEND_HANDLER => {
                    let offset = two_bytes_oper!();
                    writeln!(f, " {offset} (to {})", ip + offset)?;
                    ip += 3;
                }
                LOOP => {
                    let offset = two_bytes_oper!();
                    writeln!(f, " {offset} (back to {})", ip - offset)?;
                    ip += 3;
                }
                CLOSURE8 | CLOSURE16 => {
                    let (idx, upvaluec, size) = match instr {
                        CLOSURE8 => (byte_oper!(), byte_oper!(1), 2),
                        CLOSURE16 => (two_bytes_oper!(), byte_oper!(2), 3),
                        _ => unreachable!(),
                    };
                    let function = self.constant(idx);
                    write!(f, " {idx} ({function})")?;
                    for idx in 0..upvaluec {
                        let local = byte_oper!(idx * 2 + size) != 0;
                        let idx = byte_oper!(idx * 2 + size + 1);
                        write!(f, " {:?}", (local, idx))?;
                    }
                    write!(f, "\n")?;
                    inners.push(function);
                    ip += 1 + size + upvaluec * 2;
                }
                CALL => {
                    let argc = byte_oper!();
                    writeln!(f, " {argc}")?;
                    ip += 2;
                }
                BUILD_LIST | BUILD_HASH_MAP => {
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
        for function in inners {
            match function {
                Value::Object(Object::Function(function)) => {
                    writeln!(f, "{function}")?;
                    write!(f, "{:?}", function.chunk())?
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}
