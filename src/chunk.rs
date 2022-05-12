use super::token::Token;
use super::value::Value;
use std::{
    convert::{Into, TryFrom},
    rc::Rc,
};

#[cfg(feature = "debug-bytecode")]
use std::fmt;

#[derive(Clone, Copy)]
pub enum OpCode {
    Push,
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
    Nil,
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

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            Self::Push => 0,
            Self::Pop => 1,
            Self::Negate => 2,
            Self::Add => 3,
            Self::Subtract => 4,
            Self::Multiply => 5,
            Self::Divide => 6,
            Self::Remainder => 7,
            Self::Not => 8,
            Self::Equal => 9,
            Self::Greater => 10,
            Self::GreaterEqual => 11,
            Self::Less => 12,
            Self::LessEqual => 13,
            Self::Constant8 => 14,
            Self::Constant16 => 15,
            Self::Jump => 16,
            Self::JumpIfFalse => 17,
            Self::JumpIfTrue => 18,
            Self::Loop => 19,
            Self::Return => 20,
            Self::Nil => 21,
            Self::Call => 22,
            Self::Closure => 23,
            Self::GetLocal => 24,
            Self::SetLocal => 25,
            Self::GetUpValue => 26,
            Self::SetUpValue => 27,
            Self::CloseUpValue => 28,
            Self::GetGlobal => 29,
            Self::SetGlobal => 30,
            Self::DefineGlobal => 31,
            Self::Get => 32,
            Self::Set => 33,
            Self::BuildList => 34,
            Self::BuildObject => 35,
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Push),
            1 => Ok(Self::Pop),
            2 => Ok(Self::Negate),
            3 => Ok(Self::Add),
            4 => Ok(Self::Subtract),
            5 => Ok(Self::Multiply),
            6 => Ok(Self::Divide),
            7 => Ok(Self::Remainder),
            8 => Ok(Self::Not),
            9 => Ok(Self::Equal),
            10 => Ok(Self::Greater),
            11 => Ok(Self::GreaterEqual),
            12 => Ok(Self::Less),
            13 => Ok(Self::LessEqual),
            14 => Ok(Self::Constant8),
            15 => Ok(Self::Constant16),
            16 => Ok(Self::Jump),
            17 => Ok(Self::JumpIfFalse),
            18 => Ok(Self::JumpIfTrue),
            19 => Ok(Self::Loop),
            20 => Ok(Self::Return),
            21 => Ok(Self::Nil),
            22 => Ok(Self::Call),
            23 => Ok(Self::Closure),
            24 => Ok(Self::GetLocal),
            25 => Ok(Self::SetLocal),
            26 => Ok(Self::GetUpValue),
            27 => Ok(Self::SetUpValue),
            28 => Ok(Self::CloseUpValue),
            29 => Ok(Self::GetGlobal),
            30 => Ok(Self::SetGlobal),
            31 => Ok(Self::DefineGlobal),
            32 => Ok(Self::Get),
            33 => Ok(Self::Set),
            34 => Ok(Self::BuildList),
            35 => Ok(Self::BuildObject),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "debug-bytecode")]
impl fmt::Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:20}",
            match self {
                Self::Push => "PUSH",
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
                Self::Nil => "NIL",
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
            },
        )
    }
}

#[derive(Clone)]
pub struct Chunk<'a> {
    bytes: Vec<u8>,
    constants: Vec<Value<'a>>,
    tokens: Vec<Option<Rc<Token<'a>>>>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            constants: Vec::new(),
            tokens: Vec::new(),
        }
    }

    #[cfg(feature = "debug-bytecode")]
    fn disassemble_instr_at(&self, offset: usize) -> (String, usize) {
        let op_code = OpCode::try_from(self.bytes[offset]).unwrap();
        let mut buffer = String::new();
        buffer += format!("{:0>5} {:?}", offset, op_code).as_str();

        match op_code {
            OpCode::Push
            | OpCode::Pop
            | OpCode::Negate
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Remainder
            | OpCode::Not
            | OpCode::Equal
            | OpCode::Greater
            | OpCode::GreaterEqual
            | OpCode::Less
            | OpCode::LessEqual
            | OpCode::Return
            | OpCode::GetGlobal
            | OpCode::SetGlobal
            | OpCode::DefineGlobal
            | OpCode::Nil
            | OpCode::Get
            | OpCode::Set
            | OpCode::CloseUpValue => {
                buffer += "\n";
                return (buffer, 1);
            }
            OpCode::Constant8 => {
                let index = self.bytes[offset + 1] as usize;
                let constant = &self.constants[index];
                buffer += format!("{} ({})\n", index, constant).as_str();
                match constant {
                    Value::Function(function) => buffer += format!("{:?}", function).as_str(),
                    _ => {}
                }
                return (buffer, 2);
            }
            OpCode::Constant16 => {
                let index = ((self.bytes[offset + 2] as u16) << 8 | (self.bytes[offset + 1] as u16))
                    as usize;
                let constant = &self.constants[index];
                buffer += format!("{} ({})\n", index, constant).as_str();
                match constant {
                    Value::Function(function) => buffer += format!("{:?}", function).as_str(),
                    _ => {}
                }
                return (buffer, 3);
            }
            OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue | OpCode::Loop => {
                let offset = ((self.bytes[offset + 2] as u16) << 8
                    | (self.bytes[offset + 1] as u16)) as usize;
                buffer += format!("{}\n", offset).as_str();
                return (buffer, 3);
            }
            OpCode::Call
            | OpCode::GetLocal
            | OpCode::SetLocal
            | OpCode::GetUpValue
            | OpCode::SetUpValue
            | OpCode::BuildList
            | OpCode::BuildObject => {
                let oper = self.bytes[offset + 1] as usize;
                buffer += format!("{}\n", oper).as_str();
                return (buffer, 2);
            }
            OpCode::Closure => {
                let up_values_count = self.bytes[offset + 1] as usize;
                buffer += format!("{}\n", up_values_count).as_str();

                for i in 0..up_values_count {
                    buffer += format!(
                        "|     {i}: is_local: {}, index: {}\n",
                        self.bytes[offset + 2 + i * 2] != 0,
                        self.bytes[offset + 3 + i * 2] as usize
                    )
                    .as_str();
                }

                return (buffer, 2 + up_values_count * 2);
            }
        }
    }

    #[cfg(feature = "debug-bytecode")]
    fn disassemble(&self) -> String {
        let mut buffer = String::new();
        let mut offset = 0;
        while offset < self.len() {
            let (as_string, progress) = self.disassemble_instr_at(offset);
            buffer += &as_string;
            offset += progress;
        }
        buffer
    }

    pub fn append_instr(&mut self, op_code: OpCode, token: Option<Rc<Token<'a>>>) {
        self.bytes.push(op_code.into());
        self.tokens.push(token);
    }

    pub fn append_u16_oper(&mut self, value: u16) {
        self.bytes.push(value as u8);
        self.bytes.push((value >> 8) as u8);
    }

    pub fn append_u8_oper(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn set_u16(&mut self, index: usize, value: u16) {
        self.bytes[index] = value as u8;
        self.bytes[index + 1] = (value >> 8) as u8;
    }

    pub fn append_constant(
        &mut self,
        value: Value<'a>,
        token: Option<Rc<Token<'a>>>,
    ) -> Result<usize, ()> {
        let index = self.constants.len();
        self.constants.push(value);

        if index <= 0xff {
            self.append_instr(OpCode::Constant8, token);
            self.append_u8_oper(index as u8);
        } else if index < 0xffff {
            self.append_instr(OpCode::Constant16, token);
            self.append_u16_oper(index as u16);
        } else {
            //TODO find any way to report this error
            return Err(());
        }

        Ok(index)
    }

    // returns the index of the jump instruction
    pub fn append_jump(&mut self, op_code: OpCode, token: Option<Rc<Token<'a>>>) -> usize {
        let index = self.bytes.len();
        self.append_instr(op_code, token);
        self.append_u16_oper(0);
        index
    }

    pub fn append_loop(&mut self, loop_start: usize, token: Option<Rc<Token<'a>>>) {
        self.append_instr(OpCode::Loop, token);
        self.append_u16_oper((self.len() - 1 - loop_start) as u16); //TODO make sure that it's convertable
    }

    pub fn set_relative_jump(&mut self, index: usize) {
        self.set_u16(index + 1, (self.len() - index) as u16);
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(feature = "debug-bytecode")]
impl<'a> fmt::Debug for Chunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}
