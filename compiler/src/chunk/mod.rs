pub mod value;

use parser::token::Token;
use std::{fmt, rc::Rc};
use value::{Function, Object, Value};

/// Implements `Into<u8>` and `From<u8> for the enum created inside.
///
/// Variants aren't expected to have payloads or be more than 256.
macro_rules! byte_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname),*
        }

        impl std::convert::Into<u8> for $name {
            fn into(self) -> u8 {
                self as u8
            }
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
    pub enum OpCode {
        /// Implements `Value::neg` on TOT.
        NEG,
        /// Implements `Value::not` on TOT.
        NOT,
        /// Implmenets `Value::add` on TOT1 and TOT.
        ADD,
        /// Implements `Value::sub` on TOT1 and TOT.
        SUB,
        /// Implements `Value::mul` on TOT1 and TOT.
        MUL,
        /// Implements `Value::div` on TOT1 and TOT.
        DIV,
        /// Implements `Value::rem` on TOT1 and TOT.
        REM,
        /// Uses `Value::eq`.
        EQ,
        NOT_EQ,
        /// Implemnts `Value::partial_cmp` on TOT1 and TOT, accepting `Ordering::Greater`.
        GREATER,
        /// Implemnts `Value::partial_cmp` on TOT1 and TOT, accepting `Ordering::Greater` or `Ordering::Equal`.
        GREATER_EQ,
        /// Implemnts `Value::partial_cmp` on TOT1 and TOT, accepting `Ordering::Less`.
        LESS,
        /// Implemnts `Value::partial_cmp` on TOT1 and TOT, accepting `Ordering::Less` or `Ordering::Equal`.
        LESS_EQ,
        /// `CONST8 <idx: u8>`
        ///
        /// Pushes `constants[idx]` to tmps.
        CONST8,
        /// `CONST16 <idx: u16>`
        ///
        /// Pushes `constants[idx]` to tmps.
        CONST16,
        /// `JUMP <offset: u16>`
        ///
        /// Adds `offset` to the ip.
        JUMP,
        /// `JUMP_IF_FALSY_OR_POP <offset: u16>`
        ///
        /// Jumps if TOT is false otherwise TOT is popped.
        JUMP_IF_FALSY_OR_POP,
        /// `JUMP_IF_TRUTHY_OR_POP <offset: u16>`
        ///
        /// Jumps if TOT is true otherwise TOT is popped.
        JUMP_IF_TRUTHY_OR_POP,
        /// `POP_JUMP_IF_FALSY <offset: u16>`
        ///
        /// Jumps if TOT is false, TOT is popped.
        POP_JUMP_IF_FALSY,
        /// `POP_JUMP_IF_TRUTHY <offset: u16>`
        ///
        /// Jumps if TOT is true, TOT is popped.
        POP_JUMP_IF_TRUTHY,
        /// `FOR_ITER <offset: u16>`
        ///
        /// Keeps advancing the iterator (TOT) pushing the result to tmps until `None` is returned.
        ///
        /// When `None` is returned by the iterator it pops TOT and jumps.
        FOR_ITER,
        /// `LOOP <offset: u16>`
        ///
        /// Subtracts `offset` from the ip.
        LOOP,
        /// `GET_LOCAL <idx: u8>`
        ///
        /// Pushes `locals[frame.slots + idx]` to tmps.
        GET_LOCAL,
        /// `SET_LOCAL <idx: u8>`
        ///
        /// Sets `locals[frame.slots + idx]` to TOT, TOT stays on tmps
        SET_LOCAL,
        /// Pushes TOT to locals, TOT is popped.
        DEF_LOCAL,
        /// Pops TOL.
        POP_LOCAL,
        /// `GET_UPVALUE <idx: u8>`
        ///
        /// Pushes `frame.closure.upvalues[idx]` to tmps.
        GET_UPVALUE,
        /// `SET_UPVALUE <idx: u8>`
        ///
        /// Sets `frame.closure.upvalues[idx]` to TOT, TOT stays on tmps.
        SET_UPVALUE,
        /// Closes the upvalues tied to TOL, TOL is popped.
        CLOSE_UPVALUE,
        /// `GET_GLOBAL8 <idx: u8>`
        ///
        /// Pushes `globals[constants[idx]]` to tmps.
        ///
        /// Fails if `globals[constants[idx]]` is undefined.
        GET_GLOBAL8,
        /// `GET_GLOBAL16 <idx: u16>`
        ///
        /// Pushes `globals[constants[idx]]` to tmps.
        ///
        /// Fails if `globals[constants[idx]]` is undefined.
        GET_GLOBAL16,
        /// `SET_GLOBAL8 <idx: u8>`
        ///
        /// Sets `globals[constants[idx]]` to TOT, TOT stays on tmps.
        ///
        /// Fails if `globals[constants[idx]]` is undefined.
        SET_GLOBAL8,
        /// `SET_GLOBAL16 <idx: u16>`
        ///
        /// Sets `globals[constants[idx]]` to TOT, TOT stays on tmps.
        ///
        /// Fails if `globals[constants[idx]]` is undefined.
        SET_GLOBAL16,
        /// `DEF_GLOBAL8 <idx: u8>`
        ///
        /// Sets `globals[constants[idx]]` to TOT, TOT is popped.
        ///
        /// Fails if `globals[constants[idx]]` is already defined.
        DEF_GLOBAL8,
        /// `DEF_GLOBAL16 <idx: u16>`
        ///
        /// Sets `globals[constants[idx]]` to TOT, TOT is popped.
        ///
        /// Fails if `globals[constants[idx]]` is already defined.
        DEF_GLOBAL16,
        /// `CLOSURE8 <idx: u8> <upvaluec: u8> <(local: bool, idx: u8)>...`
        ///
        /// Expects TOT to be a `Function`, TOT gets replaced with the result.
        ///
        /// Builds a closure with TOT and the operands.
        CLOSURE8,
        /// `CLOSURE16 <idx: u16> <upvaluec: u8> <(local: bool, idx: u8)>...`
        ///
        /// Expects TOT to be a `Function`, TOT gets replaced with the result.
        ///
        /// Builds a closure with TOT and the operands.
        CLOSURE16,
        /// `CALL <argc: u8>`
        ///
        /// Expects tmps length - `argc` - 1 to be a callable, i.e, `Closure` or `Native`.
        ///
        /// Arity checks are done before executing anything.
        ///
        /// For `Closures`s it creates a frame, executes it, then pushes the returned result to tmps.
        ///
        /// For `Native`s it pops the native and its args from the tmps and invokes it with them pushing the result to tmps.
        CALL,
        /// Leaves the required and optional params and reduces the rest into a list.
        BUILD_VARIADIC,
        /// Closes any upvalue associate to one of the closure's locals, pops the locals, and returns TOT, TOT is popped.
        RET,
        /// `BUILD_LIST <size: u16>`
        ///
        /// Takes the last `size`th values from tmps and creates a list with them.
        BUILD_LIST,
        /// `BUILD_HASH_MAP <size: u16>`
        ///
        /// Expects key-value pairs to be on tmps.
        BUILD_HASH_MAP,
        /// `GET`
        ///
        /// Implements `TOT1[TOT]`, TOT and TOT1 are popped.
        ///
        /// TOT1 can be a list, string, or hash map.
        ///
        /// If TOT isn't inside TOT1, an error should be thrown.
        ///
        /// For strings and lists TOT must be an integer, but for hash maps, It must be a string.
        GET,
        /// `SET`
        ///
        /// Implements `TOT1[TOT] = TOT2`, TOT and TOT1 are popped.
        ///
        /// TOT1 can be a list or hash map.
        ///
        /// If TOT1 is a list, It should throw if TOT isn't inside.
        ///
        /// For strings and lists TOT must be an integer, but for hash maps, It must be a string.
        SET,
        /// `APPEND_HANDLER <offset: u16>`
        ///
        /// `offset` represents the difference between this instruction and the catch's block start.
        ///
        /// Appends an errors handler which contains the start of the block's slots and the ip of the catch block start.
        APPEND_HANDLER,
        /// Pops the last handler.
        POP_HANDLER,
        /// Throws TOT.
        THROW,
        /// Turns TOT into an iterator.
        ///
        /// Expects TOT to be a string or list.
        ITER,
        /// `UNPACK_LIST <to: u16>`
        ///
        /// Spreads the list into the stack.
        ///
        /// If `to` isn't equal to the length of TOT.
        ///
        /// Expects TOT to be a list.
        UNPACK_LIST,
        /// `UNPACK_HASH_MAP <propc: u16> <default: bool>...`
        ///
        /// Expects the keys and default values to be on tmps.
        ///
        /// Puts the values on tmps in the same order.
        UNPACK_HASH_MAP,
        /// Pops TOT.
        POP,
        /// Duplicates TOT.
        DUP,
        UNKNOWN,
    }
}

use OpCode::*;

#[derive(Clone)]
pub struct Instruction {
    op_code: OpCode,
    operands: Vec<u8>,
}

impl Instruction {
    fn new(op_code: OpCode, operands: &[u8]) -> Self {
        Self {
            op_code,
            operands: operands.to_owned(),
        }
    }

    /// `size` must be less than or equal to eight
    pub fn read_oper(&self, size: usize, idx: usize) -> usize {
        let operands = &self.operands[idx..idx + size];
        let mut bytes: [u8; 8] = [0; 8];
        for (i, byte) in operands.iter().enumerate() {
            bytes[i] = *byte;
        }
        usize::from_ne_bytes(bytes)
    }

    pub fn op_code(&self) -> OpCode {
        self.op_code
    }

    pub fn read_byte_oper(&self, idx: usize) -> usize {
        self.read_oper(1, idx)
    }

    pub fn read_two_bytes_oper(&self, idx: usize) -> usize {
        self.read_oper(2, idx)
    }

    pub fn size(&self) -> usize {
        1 + self.operands.len()
    }
}

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
        self.bytes.get(offset).copied()
    }

    pub fn constant(&self, idx: usize) -> Value {
        self.constants.get(idx).unwrap().clone()
    }

    pub fn token(&self, ip: usize) -> Rc<Token> {
        Rc::clone(&self.tokens[ip].as_ref().unwrap())
    }

    fn write_op_code(&mut self, op_code: OpCode, token: Rc<Token>) {
        self.bytes.push(op_code as u8);
        self.tokens.push(Some(token));
    }

    fn write_byte(&mut self, byte: usize) -> Result<(), ()> {
        if byte <= u8::MAX.into() {
            self.write_byte_unchecked(byte);
            Ok(())
        } else {
            Err(())
        }
    }

    fn write_byte_unchecked(&mut self, byte: usize) {
        self.bytes.push(byte as u8);
        self.tokens.push(None)
    }

    fn write_two_bytes(&mut self, two_bytes: usize) -> Result<(), ()> {
        if two_bytes <= u16::MAX.into() {
            let [byte1, byte2] = u16::to_ne_bytes(two_bytes as u16);
            self.write_byte(byte1 as usize).ok();
            self.write_byte(byte2 as usize).ok();
            Ok(())
        } else {
            Err(())
        }
    }

    fn rewrite_two_bytes(&mut self, idx: usize, two_bytes: usize) -> Result<(), ()> {
        if two_bytes <= u16::MAX.into() {
            let [byte1, byte2] = u16::to_ne_bytes(two_bytes as u16);
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
            value => {
                for (idx, value_) in self.constants.iter().enumerate() {
                    if value == value_ {
                        return idx;
                    }
                }
            }
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }

    /// `op_code` must be `NEG`, `NOT`, `ADD`, `SUB`, `MUL`, `DIV`, `REM`, `EQ`, `GREATER`, `GREATER_EQ`, `LESS`, `LESS_EQ`, `DEF_LOCAL`, `GET`, `SET`, `CLOSE_UPVALUE`, `BUILD_VARIADIC`, `RET`, `POP_HANDLER`, `THROW`, `ITER`, `POP`, or `DUP`.
    pub fn write_instr_no_operands(&mut self, op_code: OpCode, token: Rc<Token>) {
        self.write_op_code(op_code, token)
    }

    /// `op_code` must be `GET_LOCAL`, `SET_LOCAL`, `GET_UPVALUE`, or `SET_UPVALUE`.
    ///
    /// Fails when `idx` is greater than 255.
    pub fn write_instr_idx(
        &mut self,
        op_code: OpCode,
        token: Rc<Token>,
        idx: usize,
    ) -> Result<(), ()> {
        self.write_op_code(op_code, token);
        self.write_byte(idx)
    }

    /// `op_code` must be (`CONST8`, `CONST16`), (`GET_GLOBAL8`, `GET_GLOBAL16`), (`SET_GLOBAL8`, `SET_GLOBAL16`), (`DEF_GLOBAL8`, `DEF_GLOBAL16`), (`GET8`, `GET_16`), or (`SET8`, `SET16`).
    ///
    /// Fails when the chunk already has 65536 constants.
    pub fn write_instr_const(
        &mut self,
        (u8_op_code, u16_op_code): (OpCode, OpCode),
        token: Rc<Token>,
        value: Value,
    ) -> Result<(), ()> {
        match self.add_constant(value) {
            idx if idx <= u8::MAX.into() => {
                self.write_op_code(u8_op_code, token);
                self.write_byte(idx).ok();
                Ok(())
            }
            idx if idx <= u16::MAX.into() => {
                self.write_op_code(u16_op_code, token);
                self.write_two_bytes(idx).ok();
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// `op_code` must be `JUMP`, `POP_JUMP_IF_FALSE`, `POP_JUMP_IF_TRUE`, `JUMP_IF_FALSE_OR_POP`, `JUMP_IF_TRUE_OR_POP`, `FOR_ITER`, or `APPEND_HANDLER`.
    ///
    /// Returns its indx
    pub fn write_jump(&mut self, op_code: OpCode, token: Rc<Token>) -> usize {
        let idx = self.len();
        self.write_op_code(op_code, token);
        self.write_two_bytes(0).ok();
        idx
    }

    /// Makes the jump at `ip` point to the current ip.
    ///
    /// Fails when `self.len()` - ip is greater than 65535.
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
        self.write_op_code(LOOP, token);
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
            self.write_byte_unchecked(if local { 1 } else { 0 });
            self.write_byte_unchecked(idx);
        }
        Ok(())
    }

    /// Fails when `argc` is greater than 255.
    pub fn write_call(&mut self, token: Rc<Token>, argc: usize) -> Result<(), ()> {
        self.write_op_code(CALL, token);
        self.write_byte(argc)
    }

    /// `op_code` must be `BUILD_LIST` or `BUILD_HASH_MAP`.
    ///
    /// Fails when `size` is greater than 65535.
    pub fn write_build(
        &mut self,
        op_code: OpCode,
        token: Rc<Token>,
        size: usize,
    ) -> Result<(), ()> {
        self.write_op_code(op_code, token);
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
        self.write_op_code(UNPACK_HASH_MAP, token);
        self.write_two_bytes(defaults.len())?;
        for flag in defaults {
            self.write_byte(if flag { 1 } else { 0 })?;
        }
        Ok(())
    }

    /// Fails when `to` is greater than 65535
    pub fn write_list_unpack(&mut self, token: Rc<Token>, to: usize) -> Result<(), ()> {
        self.write_op_code(UNPACK_LIST, token);
        self.write_two_bytes(to)
    }

    pub fn read(&self, ip: usize) -> Option<Instruction> {
        macro_rules! byte_oper {
            ($($offset:expr)?) => {
                self.bytes[ip + 1$( + ($offset))?] as usize
            };
        }
        macro_rules! two_bytes_oper {
            ($($offset:expr)?) => {
                u16::from_ne_bytes([self.bytes[ip + 1$( + ($offset))?], self.bytes[ip + 2$( + ($offset))?]]) as usize
            };
        }
        macro_rules! operands {
            ($size:expr) => {
                &self.bytes[ip + 1..ip + $size]
            };
        }
        let op_code = self.byte(ip)?.into();
        match op_code {
            NEG | NOT | ADD | SUB | MUL | DIV | REM | EQ | NOT_EQ | GREATER | GREATER_EQ | LESS
            | LESS_EQ | POP_LOCAL | CLOSE_UPVALUE | BUILD_VARIADIC | RET | POP_HANDLER | THROW
            | ITER | POP | DUP | GET | SET | DEF_LOCAL => {
                Some(Instruction::new(op_code, operands!(1)))
            }
            GET_LOCAL | SET_LOCAL | GET_UPVALUE | SET_UPVALUE | CONST8 | GET_GLOBAL8
            | SET_GLOBAL8 | DEF_GLOBAL8 | CALL => Some(Instruction::new(op_code, operands!(2))),
            CONST16
            | GET_GLOBAL16
            | SET_GLOBAL16
            | DEF_GLOBAL16
            | JUMP
            | POP_JUMP_IF_FALSY
            | POP_JUMP_IF_TRUTHY
            | JUMP_IF_FALSY_OR_POP
            | JUMP_IF_TRUTHY_OR_POP
            | FOR_ITER
            | APPEND_HANDLER
            | LOOP
            | BUILD_LIST
            | BUILD_HASH_MAP
            | UNPACK_LIST => Some(Instruction::new(op_code, operands!(3))),
            UNPACK_HASH_MAP => Some(Instruction::new(op_code, operands!(3 + two_bytes_oper!()))),
            CLOSURE8 => Some(Instruction::new(op_code, operands!(3 + byte_oper!(1) * 2))),
            CLOSURE16 => Some(Instruction::new(op_code, operands!(4 + byte_oper!(2) * 2))),
            UNKNOWN => unreachable!(),
        }
    }

    fn disassemble_instr(&self, ip: usize) -> Option<(String, usize)> {
        let instr = self.read(ip)?;
        let token = self.token(ip);
        let mut buf = String::new();
        buf += format!("{:>5} {:20}", ip, format!("{:?}", instr.op_code())).as_str();
        match instr.op_code() {
            NEG | NOT | ADD | SUB | MUL | DIV | REM | EQ | NOT_EQ | GREATER | GREATER_EQ | LESS
            | LESS_EQ | POP_LOCAL | CLOSE_UPVALUE | BUILD_VARIADIC | RET | POP_HANDLER | THROW
            | ITER | POP | DUP | GET | SET => {}
            DEF_LOCAL => buf += format!(" ({})", token.lexeme()).as_str(),
            GET_LOCAL | SET_LOCAL | GET_UPVALUE | SET_UPVALUE => {
                buf += format!(" {} ({})", instr.read_byte_oper(0), token.lexeme()).as_str()
            }
            CONST8 | GET_GLOBAL8 | SET_GLOBAL8 | DEF_GLOBAL8 => {
                let idx = instr.read_byte_oper(0);
                buf += format!(" {idx} ({})", self.constant(idx)).as_str()
            }
            CONST16 | GET_GLOBAL16 | SET_GLOBAL16 | DEF_GLOBAL16 => {
                let idx = instr.read_two_bytes_oper(0);
                buf += format!(" {idx} ({})", self.constant(idx)).as_str()
            }
            JUMP
            | POP_JUMP_IF_FALSY
            | POP_JUMP_IF_TRUTHY
            | JUMP_IF_FALSY_OR_POP
            | JUMP_IF_TRUTHY_OR_POP
            | FOR_ITER
            | APPEND_HANDLER => {
                let offset = instr.read_two_bytes_oper(0);
                buf += format!(" {offset} (to {})", ip + offset).as_str()
            }
            LOOP => {
                let offset = instr.read_two_bytes_oper(0);
                buf += format!(" {offset} (back to {})", ip - offset).as_str()
            }
            CLOSURE8 | CLOSURE16 => {
                let (idx, upvaluec, size) = match instr.op_code() {
                    CLOSURE8 => (instr.read_byte_oper(0), instr.read_byte_oper(1), 2),
                    CLOSURE16 => (instr.read_two_bytes_oper(0), instr.read_byte_oper(2), 3),
                    _ => unreachable!(),
                };
                let function = self.constant(idx);
                buf += format!(" {idx} ({function})").as_str();
                for idx in 0..upvaluec {
                    let local = instr.read_byte_oper(size + idx * 2) != 0;
                    let idx = instr.read_byte_oper(size + idx * 2 + 1);
                    buf += format!(" {:?}", (local, idx)).as_str()
                }
            }
            CALL => {
                let argc = instr.read_byte_oper(0);
                buf += format!(" {argc}").as_str()
            }
            BUILD_LIST | BUILD_HASH_MAP => {
                let size = instr.read_two_bytes_oper(0);
                buf += format!(" {size}").as_str()
            }
            UNPACK_HASH_MAP => {
                let propc = instr.read_two_bytes_oper(0);
                for idx in 0..propc {
                    let default = instr.read_byte_oper(2 + idx) != 0;
                    buf += format!(" {default}").as_str()
                }
            }
            UNPACK_LIST => {
                let to = instr.read_two_bytes_oper(0);
                buf += format!(" {to}").as_str()
            }
            UNKNOWN => unreachable!(),
        }
        Some((buf, instr.size()))
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ip = 0;
        if let Some((instr, size)) = self.disassemble_instr(ip) {
            let (mut line, _) = self.token(ip).pos();
            write!(f, "{line:<5} {instr}")?;
            ip += size;
            while let Some((instr, size)) = self.disassemble_instr(ip) {
                let pos = self.token(ip).pos();
                if pos.0 != line {
                    (line, _) = pos;
                    write!(f, "\n{line:<5} ")?
                } else {
                    write!(f, "\n{:5} ", "")?
                }
                write!(f, "{instr}")?;
                ip += size
            }
        }
        for constant in &self.constants {
            match constant {
                Value::Object(Object::Function(function)) => {
                    writeln!(f, "\n[CHUNK] {function}'s chunk")?;
                    write!(f, "{:?}", function.chunk())?
                }
                _ => {}
            }
        }
        Ok(())
    }
}
