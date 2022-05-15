use super::token::Token;
use super::value::Value;
use std::{
    convert::{Into, TryFrom},
    rc::Rc,
};

#[cfg(feature = "debug-bytecode")]
use std::fmt;

#[derive(Clone, Copy)]
pub enum Instruction {
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

impl Into<u8> for Instruction {
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

impl TryFrom<u8> for Instruction {
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
impl fmt::Debug for Instruction {
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

const NIL_CONST: usize = 0;
const TRUE_CONST: usize = 1;
const FALSE_CONST: usize = 2;

#[derive(Clone)]
pub struct Chunk<'a> {
    bytes: Vec<u8>,
    constants: Vec<Value<'a>>,
    tokens: Vec<Option<Rc<Token<'a>>>>,
}

impl<'a> Chunk<'a> {
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

    #[cfg(feature = "debug-bytecode")]
    fn disassemble_instr_at(&self, offset: usize) -> (String, usize) {
        let instr = Instruction::try_from(self.bytes[offset]).unwrap();
        let mut buffer = String::new();
        buffer += format!("{:0>5} {:?}", offset, instr).as_str();

        match instr {
            Instruction::Push
            | Instruction::Pop
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
            | Instruction::Nil
            | Instruction::Get
            | Instruction::Set
            | Instruction::CloseUpValue => {
                buffer += "\n";
                return (buffer, 1);
            }
            Instruction::Constant8 => {
                let index = self.bytes[offset + 1] as usize;
                let constant = &self.constants[index];
                buffer += format!("{} ({})\n", index, constant).as_str();
                match constant {
                    Value::Function(function) => buffer += format!("{:?}", function).as_str(),
                    _ => {}
                }
                return (buffer, 2);
            }
            Instruction::Constant16 => {
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
            Instruction::Jump
            | Instruction::JumpIfFalse
            | Instruction::JumpIfTrue
            | Instruction::Loop => {
                let offset = ((self.bytes[offset + 2] as u16) << 8
                    | (self.bytes[offset + 1] as u16)) as usize;
                buffer += format!("{}\n", offset).as_str();
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

    pub fn emit_instr(&mut self, instr: Instruction, token: Option<Rc<Token<'a>>>) {
        self.bytes.push(instr.into());
        self.tokens.push(token);
    }

    pub fn emit_bytes(&mut self, value: u16) {
        self.bytes.push(value as u8);
        self.bytes.push((value >> 8) as u8);
    }

    pub fn emit_byte(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn patch_bytes(&mut self, index: usize, value: u16) {
        self.bytes[index] = value as u8;
        self.bytes[index + 1] = (value >> 8) as u8;
    }

    fn add_constant(&mut self, value: Value<'a>) -> Result<usize, ()> {
        match &value {
            Value::Nil => return Ok(NIL_CONST),
            Value::Bool(val) => return Ok(if *val { TRUE_CONST } else { FALSE_CONST }),
            Value::String(string) => {
                for (index, const_) in self.constants.iter().enumerate() {
                    if let Value::String(string_2) = const_ {
                        if string_2 == string {
                            return Ok(index);
                        }
                    }
                }
            }
            _ => {}
        }

        let index = self.constants.len();
        self.constants.push(value);

        Ok(index)
    }

    pub fn emit_const(
        &mut self,
        value: Value<'a>,
        token: Option<Rc<Token<'a>>>,
    ) -> Result<usize, ()> {
        let index = self.add_constant(value)?;

        if index <= 0xff {
            self.emit_instr(Instruction::Constant8, token);
            self.emit_byte(index as u8);
        } else if index < 0xffff {
            self.emit_instr(Instruction::Constant16, token);
            self.emit_bytes(index as u16);
        } else {
            //TODO find any way to report this error
            return Err(());
        }

        Ok(index)
    }

    // returns the index of the jump instruction
    pub fn emit_jump(&mut self, instr: Instruction, token: Option<Rc<Token<'a>>>) -> usize {
        let index = self.bytes.len();
        self.emit_instr(instr, token);
        self.emit_bytes(0);
        index
    }

    pub fn emit_loop(&mut self, loop_start: usize, token: Option<Rc<Token<'a>>>) {
        self.emit_instr(Instruction::Loop, token);
        self.emit_bytes((self.len() - 1 - loop_start) as u16); //TODO make sure that it's convertable
    }

    pub fn patch_jump(&mut self, index: usize) {
        self.patch_bytes(index + 1, (self.len() - index) as u16);
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
