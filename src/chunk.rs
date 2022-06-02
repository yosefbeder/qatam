use super::{
    token::Token,
    utils::{combine, split},
    value::Value,
};
use std::{
    convert::{Into, TryFrom},
    fmt,
    rc::Rc,
};

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
}

impl Into<u8> for Instruction {
    fn into(self) -> u8 {
        match self {
            Instruction::Pop => 0,
            Instruction::Negate => 1,
            Instruction::Add => 2,
            Instruction::Subtract => 3,
            Instruction::Multiply => 4,
            Instruction::Divide => 5,
            Instruction::Remainder => 6,
            Instruction::Not => 7,
            Instruction::Equal => 8,
            Instruction::Greater => 9,
            Instruction::GreaterEqual => 10,
            Instruction::Less => 11,
            Instruction::LessEqual => 12,
            Instruction::Constant8 => 13,
            Instruction::Constant16 => 14,
            Instruction::Jump => 15,
            Instruction::JumpIfFalse => 16,
            Instruction::JumpIfTrue => 17,
            Instruction::Loop => 18,
            Instruction::Return => 19,
            Instruction::Call => 20,
            Instruction::Closure => 21,
            Instruction::GetLocal => 22,
            Instruction::SetLocal => 23,
            Instruction::GetUpValue => 24,
            Instruction::SetUpValue => 25,
            Instruction::CloseUpValue => 26,
            Instruction::GetGlobal => 27,
            Instruction::SetGlobal => 28,
            Instruction::DefineGlobal => 29,
            Instruction::Get => 30,
            Instruction::Set => 31,
            Instruction::BuildList => 32,
            Instruction::BuildObject => 33,
        }
    }
}

impl TryFrom<u8> for Instruction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Pop),
            1 => Ok(Instruction::Negate),
            2 => Ok(Instruction::Add),
            3 => Ok(Instruction::Subtract),
            4 => Ok(Instruction::Multiply),
            5 => Ok(Instruction::Divide),
            6 => Ok(Instruction::Remainder),
            7 => Ok(Instruction::Not),
            8 => Ok(Instruction::Equal),
            9 => Ok(Instruction::Greater),
            10 => Ok(Instruction::GreaterEqual),
            11 => Ok(Instruction::Less),
            12 => Ok(Instruction::LessEqual),
            13 => Ok(Instruction::Constant8),
            14 => Ok(Instruction::Constant16),
            15 => Ok(Instruction::Jump),
            16 => Ok(Instruction::JumpIfFalse),
            17 => Ok(Instruction::JumpIfTrue),
            18 => Ok(Instruction::Loop),
            19 => Ok(Instruction::Return),
            20 => Ok(Instruction::Call),
            21 => Ok(Instruction::Closure),
            22 => Ok(Instruction::GetLocal),
            23 => Ok(Instruction::SetLocal),
            24 => Ok(Instruction::GetUpValue),
            25 => Ok(Instruction::SetUpValue),
            26 => Ok(Instruction::CloseUpValue),
            27 => Ok(Instruction::GetGlobal),
            28 => Ok(Instruction::SetGlobal),
            29 => Ok(Instruction::DefineGlobal),
            30 => Ok(Instruction::Get),
            31 => Ok(Instruction::Set),
            32 => Ok(Instruction::BuildList),
            33 => Ok(Instruction::BuildObject),
            _ => Err(()),
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = match self {
            Self::Pop => "POP",
            Self::Negate => "NEGATE",
            Self::Add => "ADD",
            Self::Subtract => "SUBTRACT",
            Self::Multiply => "MULTIPLY",
            Self::Divide => "DIVIDE",
            Self::Remainder => "REMAINDER",
            Self::Not => "NOT",
            Self::Equal => "EQUAL",
            Self::Greater => "GREATER",
            Self::GreaterEqual => "GREATER_EQUAL",
            Self::Less => "LESS",
            Self::LessEqual => "LESS_EQUAL",
            Self::Constant8 => "CONSTANT8",
            Self::Constant16 => "CONSTANT16",
            Self::Jump => "JUMP",
            Self::JumpIfFalse => "JUMP_IF_FALSE",
            Self::JumpIfTrue => "JUMP_IF_TRUE",
            Self::Loop => "LOOP",
            Self::Return => "RETURN",
            Self::Call => "CALL",
            Self::Closure => "CLOSURE",
            Self::GetLocal => "GET_LOCAL",
            Self::SetLocal => "SET_LOCAL",
            Self::GetUpValue => "GET_UPVALUE",
            Self::SetUpValue => "SET_UPVALUE",
            Self::CloseUpValue => "CLOSE_UPVALUE",
            Self::GetGlobal => "GET_GLOBAL",
            Self::SetGlobal => "SET_GLOBAL",
            Self::DefineGlobal => "DEFINE_GLOBAL",
            Self::Get => "GET",
            Self::Set => "SET",
            Self::BuildList => "BUILD_LIST",
            Self::BuildObject => "BUILD_OBJECT",
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
            Instruction::Pop
            | Instruction::Negate
            | Instruction::Add
            | Instruction::Subtract
            | Instruction::Multiply
            | Instruction::Divide
            | Instruction::Remainder
            | Instruction::Not
            | Instruction::Equal
            | Instruction::Greater
            | Instruction::GreaterEqual
            | Instruction::Less
            | Instruction::LessEqual
            | Instruction::Return
            | Instruction::GetGlobal
            | Instruction::SetGlobal
            | Instruction::DefineGlobal
            | Instruction::Get
            | Instruction::Set
            | Instruction::CloseUpValue => {
                buffer += "\n";
                return (buffer, 1);
            }
            Instruction::Constant8 => {
                let idx = self.bytes[offset + 1] as usize;
                let constant = &self.constants[idx];
                buffer += format!("{} ({})\n", idx, constant).as_str();
                if expand_inner_chunks {
                    match constant {
                        Value::Function(function) => buffer += format!("{:?}", function).as_str(),
                        _ => {}
                    }
                }
                return (buffer, 2);
            }
            Instruction::Constant16 => {
                let idx = combine(self.bytes[offset + 1], self.bytes[offset + 2]) as usize;
                let constant = &self.constants[idx];
                buffer += format!("{} ({})\n", idx, constant).as_str();
                if expand_inner_chunks {
                    match constant {
                        Value::Function(function) => buffer += format!("{:?}", function).as_str(),
                        _ => {}
                    }
                }
                return (buffer, 3);
            }
            Instruction::Jump
            | Instruction::JumpIfFalse
            | Instruction::JumpIfTrue
            | Instruction::Loop => {
                let size = combine(self.bytes[offset + 1], self.bytes[offset + 2]) as usize;
                buffer += format!("{}\n", size).as_str();
                return (buffer, 3);
            }
            Instruction::Call
            | Instruction::GetLocal
            | Instruction::SetLocal
            | Instruction::GetUpValue
            | Instruction::SetUpValue
            | Instruction::BuildList
            | Instruction::BuildObject => {
                let oper = self.bytes[offset + 1] as usize;
                buffer += format!("{}\n", oper).as_str();
                return (buffer, 2);
            }
            Instruction::Closure => {
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

    pub fn emit_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
        self.tokens.push(None);
    }

    pub fn emit_instr(&mut self, instr: Instruction, token: Option<Rc<Token>>) {
        self.bytes.push(instr.into());
        self.tokens.push(token);
    }

    pub fn emit_bytes(&mut self, bytes: u16) {
        split(bytes)
            .into_iter()
            .for_each(|byte| self.emit_byte(byte));
    }

    fn patch_bytes(&mut self, idx: usize, bytes: u16) {
        self.bytes[idx] = bytes as u8;
        self.bytes[idx + 1] = (bytes >> 8) as u8;
    }

    fn add_constant(&mut self, value: Value) -> Result<usize, ()> {
        match &value {
            Value::Nil => return Ok(NIL_CONST),
            Value::Bool(val) => return Ok(if *val { TRUE_CONST } else { FALSE_CONST }),
            Value::String(string) => {
                for (idx, const_) in self.constants.iter().enumerate() {
                    if let Value::String(string_2) = const_ {
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

    pub fn emit_const(&mut self, value: Value, token: Option<Rc<Token>>) -> Result<usize, ()> {
        let idx = self.add_constant(value)?;

        if idx <= 0xff {
            self.emit_instr(Instruction::Constant8, token);
            self.emit_byte(idx as u8);
        } else if idx < 0xffff {
            self.emit_instr(Instruction::Constant16, token);
            self.emit_bytes(idx as u16);
        } else {
            //TODO find any way to report this error
            return Err(());
        }

        Ok(idx)
    }

    // returns the idx of the jump instruction
    pub fn emit_jump(&mut self, instr: Instruction, token: Option<Rc<Token>>) -> usize {
        let idx = self.bytes.len();
        self.emit_instr(instr, token);
        self.emit_bytes(0);
        idx
    }

    //TODO have another look at usize -> u16 conversions
    pub fn emit_loop(&mut self, start: usize, token: Option<Rc<Token>>) {
        self.emit_instr(Instruction::Loop, token);
        let size = self.len() - 1 - start;
        self.emit_bytes(size as u16);
    }

    pub fn patch_jump(&mut self, idx: usize) {
        let size = self.len() - idx;
        self.patch_bytes(idx + 1, size as u16);
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
