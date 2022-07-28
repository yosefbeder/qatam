use super::value::{Function, Object, Value};
use super::UpValue;
use lexer::token::Token;

use std::{
    convert::{From, Into},
    fmt,
    rc::Rc,
};

fn combine(a: u8, b: u8) -> u16 {
    (b as u16) << 8 | (a as u16)
}

fn split(bytes: u16) -> [u8; 2] {
    [bytes as u8, (bytes >> 8) as u8]
}

#[derive(Clone, Copy)]
pub enum Instruction {
    Pop,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Not,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Constant8,
    Constant16,
    Jump,
    JumpIfFalse,
    JumpIfTrue,
    Loop,
    Return,
    Call,
    Closure,
    GetLocal,
    SetLocal,
    GetUpValue,
    SetUpValue,
    CloseUpValue,
    GetGlobal,
    SetGlobal,
    DefineGlobal,
    Get,
    Set,
    BuildList,
    BuildObject,
    AppendHandler,
    PopHandler,
    Throw,
    Size,
    UnpackList,
    UnpackObject,
    PushTmp,
    FlushTmps,
    CloneTop,
    Unknown,
}

use Instruction::*;

impl Into<u8> for Instruction {
    fn into(self) -> u8 {
        match self {
            Pop => 0,
            Negate => 1,
            Add => 2,
            Subtract => 3,
            Multiply => 4,
            Divide => 5,
            Remainder => 6,
            Not => 7,
            Equal => 8,
            Greater => 9,
            GreaterEqual => 10,
            Less => 11,
            LessEqual => 12,
            Constant8 => 13,
            Constant16 => 14,
            Jump => 15,
            JumpIfFalse => 16,
            JumpIfTrue => 17,
            Loop => 18,
            Return => 19,
            Call => 20,
            Closure => 21,
            GetLocal => 22,
            SetLocal => 23,
            GetUpValue => 24,
            SetUpValue => 25,
            CloseUpValue => 26,
            GetGlobal => 27,
            SetGlobal => 28,
            DefineGlobal => 29,
            Get => 30,
            Set => 31,
            BuildList => 32,
            BuildObject => 33,
            AppendHandler => 34,
            PopHandler => 35,
            Throw => 36,
            Size => 37,
            UnpackList => 38,
            UnpackObject => 39,
            PushTmp => 40,
            FlushTmps => 41,
            CloneTop => 42,
            Unknown => 43,
        }
    }
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        match value {
            0 => Pop,
            1 => Negate,
            2 => Add,
            3 => Subtract,
            4 => Multiply,
            5 => Divide,
            6 => Remainder,
            7 => Not,
            8 => Equal,
            9 => Greater,
            10 => GreaterEqual,
            11 => Less,
            12 => LessEqual,
            13 => Constant8,
            14 => Constant16,
            15 => Jump,
            16 => JumpIfFalse,
            17 => JumpIfTrue,
            18 => Loop,
            19 => Return,
            20 => Call,
            21 => Closure,
            22 => GetLocal,
            23 => SetLocal,
            24 => GetUpValue,
            25 => SetUpValue,
            26 => CloseUpValue,
            27 => GetGlobal,
            28 => SetGlobal,
            29 => DefineGlobal,
            30 => Get,
            31 => Set,
            32 => BuildList,
            33 => BuildObject,
            34 => AppendHandler,
            35 => PopHandler,
            36 => Throw,
            37 => Size,
            38 => UnpackList,
            39 => UnpackObject,
            40 => PushTmp,
            41 => FlushTmps,
            42 => CloneTop,
            _ => Unknown,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = match self {
            Pop => "POP",
            Negate => "NEGATE",
            Add => "ADD",
            Subtract => "SUBTRACT",
            Multiply => "MULTIPLY",
            Divide => "DIVIDE",
            Remainder => "REMAINDER",
            Not => "NOT",
            Equal => "EQUAL",
            Greater => "GREATER",
            GreaterEqual => "GREATER_EQUAL",
            Less => "LESS",
            LessEqual => "LESS_EQUAL",
            Constant8 => "CONSTANT8",
            Constant16 => "CONSTANT16",
            Jump => "JUMP",
            JumpIfFalse => "JUMP_IF_FALSE",
            JumpIfTrue => "JUMP_IF_TRUE",
            Loop => "LOOP",
            Return => "RETURN",
            Call => "CALL",
            Closure => "CLOSURE",
            GetLocal => "GET_LOCAL",
            SetLocal => "SET_LOCAL",
            GetUpValue => "GET_UPVALUE",
            SetUpValue => "SET_UPVALUE",
            CloseUpValue => "CLOSE_UPVALUE",
            GetGlobal => "GET_GLOBAL",
            SetGlobal => "SET_GLOBAL",
            DefineGlobal => "DEFINE_GLOBAL",
            Get => "GET",
            Set => "SET",
            BuildList => "BUILD_LIST",
            BuildObject => "BUILD_OBJECT",
            AppendHandler => "APPEND_HANDLER",
            PopHandler => "POP_HANDLER",
            Throw => "THROW",
            Size => "SIZE",
            UnpackList => "UNPACK_LIST",
            UnpackObject => "UNPACK_OBJECT",
            PushTmp => "PUSH_TMP",
            FlushTmps => "FLUSH_TMPS",
            CloneTop => "CLONE_TOP",
            Unknown => "UNKNOWN",
        };

        if let Some(width) = f.width() {
            write!(f, "{:width$}", content)
        } else {
            write!(f, "{}", content)
        }
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
        let mut chunk = Self {
            bytes: Vec::new(),
            constants: Vec::new(),
            tokens: Vec::new(),
        };

        //? the order here is important
        chunk.constants.push(Value::Nil);
        chunk.constants.push(Value::Bool(true));
        chunk.constants.push(Value::Bool(false));

        chunk
    }

    pub fn disassemble_instr_at(
        &self,
        offset: usize,
        expand_inner_chunks: bool,
    ) -> (String, usize) {
        let instr = Instruction::try_from(self.bytes[offset]).unwrap();
        let mut buffer = String::new();
        buffer += format!("{:0>5} {:20?}", offset, instr).as_str();

        match instr {
            Pop | Negate | Add | Subtract | Multiply | Divide | Remainder | Not | Equal
            | Greater | GreaterEqual | Less | LessEqual | Return | GetGlobal | SetGlobal
            | DefineGlobal | Get | Set | CloseUpValue | PopHandler | Throw | Size | PushTmp
            | FlushTmps | CloneTop => {
                buffer += "\n";
                return (buffer, 1);
            }
            Constant8 => {
                let idx = self.bytes[offset + 1] as usize;
                let constant = &self.constants[idx];
                buffer += format!("{} ({})\n", idx, constant).as_str();
                if expand_inner_chunks {
                    match constant {
                        Value::Object(Object::Function(function)) => {
                            buffer += format!("{:?}", function).as_str()
                        }
                        _ => {}
                    }
                }
                return (buffer, 2);
            }
            Constant16 => {
                let idx = combine(self.bytes[offset + 1], self.bytes[offset + 2]) as usize;
                let constant = &self.constants[idx];
                buffer += format!("{} ({})\n", idx, constant).as_str();
                if expand_inner_chunks {
                    match constant {
                        Value::Object(Object::Function(function)) => {
                            buffer += format!("{:?}", function).as_str()
                        }
                        _ => {}
                    }
                }
                return (buffer, 3);
            }
            Jump | JumpIfFalse | JumpIfTrue | Loop | AppendHandler => {
                let size = combine(self.bytes[offset + 1], self.bytes[offset + 2]) as usize;
                buffer += format!("{}\n", size).as_str();
                return (buffer, 3);
            }
            Call | GetLocal | SetLocal | GetUpValue | SetUpValue | BuildList | BuildObject
            | UnpackList => {
                let oper = self.bytes[offset + 1] as usize;
                buffer += format!("{}\n", oper).as_str();
                return (buffer, 2);
            }
            Closure => {
                let up_values_count = self.bytes[offset + 1] as usize;
                buffer += format!("{}\n", up_values_count).as_str();

                for i in 0..up_values_count {
                    buffer += format!(
                        "|     {i}: is_local: {}, idx: {}\n",
                        self.bytes[offset + 2 + i * 2] != 0,
                        self.bytes[offset + 3 + i * 2] as usize
                    )
                    .as_str();
                }

                return (buffer, 2 + up_values_count * 2);
            }
            UnpackObject => {
                let len = self.bytes[offset + 1] as usize;
                let mut has_default = vec![];
                for idx in 0..len {
                    has_default.push(self.bytes[offset + idx + 2]);
                }
                buffer += format!("{len} {has_default:?}\n").as_str();
                return (buffer, 2 + len);
            }
            Unknown => unreachable!(),
        }
    }

    fn disassemble(&self) -> String {
        let mut buffer = String::new();
        let mut offset = 0;
        while offset < self.len() {
            let (as_string, progress) = self.disassemble_instr_at(offset, true);
            buffer += &as_string;
            offset += progress;
        }
        buffer
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
        self.tokens.push(None);
    }

    pub fn write_instr(&mut self, instr: Instruction, token: Rc<Token>) {
        self.bytes.push(instr.into());
        self.tokens.push(Some(token));
    }

    pub fn write_2bytes(&mut self, bytes: u16) {
        split(bytes)
            .into_iter()
            .for_each(|byte| self.write_byte(byte));
    }

    fn rewrite_2bytes(&mut self, idx: usize, bytes: u16) {
        self.bytes[idx] = bytes as u8;
        self.bytes[idx + 1] = (bytes >> 8) as u8;
    }

    fn add_constant(&mut self, value: Value) -> Result<usize, ()> {
        match &value {
            Value::Nil => return Ok(NIL_CONST),
            Value::Bool(val) => return Ok(if *val { TRUE_CONST } else { FALSE_CONST }),
            Value::Object(Object::String(string)) => {
                for (idx, const_) in self.constants.iter().enumerate() {
                    if let Value::Object(Object::String(string_2)) = const_ {
                        if string_2 == string {
                            return Ok(idx);
                        }
                    }
                }
            }
            _ => {}
        }

        let idx = self.constants.len();
        self.constants.push(value);

        Ok(idx)
    }

    pub fn write_const(&mut self, value: Value, token: Rc<Token>) -> Result<usize, ()> {
        let idx = self.add_constant(value)?;

        if idx <= 0xff {
            self.write_instr(Constant8, token);
            self.write_byte(idx as u8);
        } else if idx < 0xffff {
            self.write_instr(Constant16, token);
            self.write_2bytes(idx as u16);
        } else {
            return Err(());
        }

        Ok(idx)
    }

    // returns the idx of the jump instruction
    pub fn write_jump(&mut self, instr: Instruction, token: Rc<Token>) -> usize {
        let idx = self.bytes.len();
        self.write_instr(instr, token);
        self.write_2bytes(0);
        idx
    }

    pub fn write_closure(
        &mut self,
        function: Function,
        up_values: &[UpValue],
        token: Rc<Token>,
    ) -> Result<(), ()> {
        self.write_const(Value::new_function(function), token.clone())?;
        self.write_instr(Closure, token);
        self.write_byte(up_values.len() as u8);
        for up_value in up_values {
            self.write_byte(up_value.is_local as u8);
            self.write_byte(up_value.idx as u8);
        }
        Ok(())
    }

    pub fn write_loop(&mut self, start: usize, token: Rc<Token>) {
        self.write_instr(Loop, token);
        let size = self.len() - 1 - start;
        self.write_2bytes(size as u16);
    }

    pub fn rewrite_jump(&mut self, idx: usize) {
        let size = self.len() - idx;
        self.rewrite_2bytes(idx + 1, size as u16);
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(offset).cloned()
    }

    pub fn get_constant(&self, idx: usize) -> Value {
        self.constants.get(idx).unwrap().clone()
    }

    pub fn get_token(&self, idx: usize) -> Rc<Token> {
        Rc::clone(&self.tokens.get(idx).unwrap().as_ref().unwrap())
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}
