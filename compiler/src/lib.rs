pub mod chunk;

use chunk::value::Value;
use chunk::Chunk;
use chunk::Instruction::{self, *};
use colored::Colorize;
use lexer::token::{Token, TokenType};
use parser::ast::{Expr, Literal, Stml};
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug, Clone)]
pub enum CompileError {
    TooManyConsts(Rc<Token>),
    HugeSize(Rc<Token>),
    BackSlashMisuse(Rc<Token>),
    DefaultInObject(Rc<Token>),
    HugeJump(Rc<Token>),
    TooManyLocals(Rc<Token>),
    TooManyUpvalues(Rc<Token>),
    SameVarInScope(Rc<Token>),
    InvalidDes(Rc<Token>),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "خطأ ترجمي: ".bright_red())?;
        match self {
            TooManyConsts(token) => {
                writeln!(f, "لا يمكن أن تحتوي الدالة الواحدة على أكثر من 65536  ثابت")?;
                write!(f, "{token}")
            }
            HugeSize(token) => {
                write!(f, "لا يمكن أن  ")?;
                if token.typ == TokenType::OBracket {
                    write!(f, "تنشأ قائمة جديدة ")?
                } else {
                    write!(f, "ينشأ كائن جديد ")?
                }
                writeln!(f, "بأكثر من 65535 عنصر")?;
                write!(f, "{token}")
            }
            BackSlashMisuse(token) => {
                writeln!(f, "استعمال خاطئ ل\"\\\"")?;
                writeln!(f, "{token}")?;
                write!(
                    f,
                    "حيث يمكن أن تكون متلية فقط ب\"n\" أو \"r\" أو \"t\" أو '\"'"
                )
            }
            DefaultInObject(token) => {
                writeln!(
                    f,
                    "لا يمكن أن يحتوي كائن على قيمة إفتراضية - حيث أنها تكون فقط في التوزيع -"
                )?;
                write!(f, "{token}")
            }
            HugeJump(_) => {
                // TODO find a good msg
                todo!()
            }
            TooManyLocals(token) => {
                writeln!(f, "لا يمكن أن تحتوي دالة على أكثر من 256 متغير خاص")?;
                write!(f, "{token}")
            }
            TooManyUpvalues(token) => {
                writeln!(
                    f,
                    "لا يمكن لدالة أن تشير إلى أكثر من 256 متغير من دوال مغلقة عليها"
                )?;
                write!(f, "{token}")
            }
            SameVarInScope(token) => {
                writeln!(f, "يوجد متغير يسمى \"{}\" في نفس المجموعة", token.lexeme)?;
                write!(f, "{token}")
            }
            InvalidDes(token) => {
                writeln!(f, "يمكن فقط استخدام الكلمات والقوائم والكائنات في التوزيع")?;
                write!(f, "{token}")
            }
        }
    }
}

use CompileError::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DesType {
    Define,
    Set,
}

#[derive(Debug, Clone)]
struct Locals {
    /// name, depth
    inner: Vec<(Rc<Token>, usize)>,
    /// local, idx
    upvalues: Vec<(bool, usize)>,
    depth: usize,
    enclosing: Option<Rc<RefCell<Locals>>>,
}

impl Locals {
    fn new(enclosing: Option<Rc<RefCell<Locals>>>) -> Self {
        Self {
            inner: Vec::with_capacity(256),
            upvalues: Vec::with_capacity(256),
            depth: 0,
            enclosing,
        }
    }

    /// Fails when `self.inner` is larger than 256.
    fn push(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if self.inner.capacity() == self.inner.len() {
            Err(())
        } else {
            self.inner.push((token, self.depth));
            Ok(())
        }
    }

    fn get(&self, idx: usize) -> Option<&(Rc<Token>, usize)> {
        self.inner.get(idx)
    }

    /// Fails when `self.upvalues` is larger than 256.
    fn add_upvalue(&mut self, local: bool, idx: usize) -> Result<usize, ()> {
        for (idx, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.0 == local && upvalue.1 == idx {
                return Ok(idx);
            }
        }
        if self.upvalues.capacity() == self.inner.len() {
            Err(())
        } else {
            let idx = self.upvalues.len();
            self.upvalues.push((local, idx));
            Ok(idx)
        }
    }

    /// `token` must be of type `Identifier`.
    fn resolve_upvalue(&mut self, token: Rc<Token>) -> Result<Option<usize>, ()> {
        match self.enclosing.clone() {
            Some(enclosing) => {
                if let Some(idx) = enclosing.borrow().resolve_local(Rc::clone(&token)) {
                    Ok(Some(self.add_upvalue(true, idx)?))
                } else {
                    enclosing.borrow_mut().resolve_upvalue(token)
                }
            }
            None => Ok(None),
        }
    }

    /// `token` must be of type `Identifier`.
    fn resolve_local(&self, token: Rc<Token>) -> Option<usize> {
        for (idx, (name, _)) in self.inner.iter().enumerate().rev() {
            if name.lexeme == token.lexeme {
                return Some(idx);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompilerType {
    Script,
    /// Writes an implicit `Nil` return at the end of the chunk.
    Function,
    /// Writes an implicit return for exported functions.
    Module,
}

pub struct Compiler<'a> {
    typ: CompilerType,
    ast: &'a Vec<Stml>,
    /// The token that represents the compiler.
    ///
    /// It must be of type `OBrace` for a function compiler and `EOF` for the rest.
    token: Rc<Token>,
    chunk: Chunk,
    locals: Rc<RefCell<Locals>>,
    depth: usize,
    errors: RefCell<Vec<CompileError>>,
}

impl<'a> Compiler<'a> {
    pub fn new(typ: CompilerType, ast: &'a Vec<Stml>, token: Rc<Token>) -> Self {
        Self {
            typ,
            ast,
            token,
            chunk: Chunk::new(),
            locals: Rc::new(RefCell::new(Locals::new(None))),
            depth: 0,
            errors: RefCell::new(vec![]),
        }
    }

    fn err(&self, err: CompileError) {
        self.errors.borrow_mut().push(err)
    }

    fn global(&self) -> bool {
        // TODO change when you add compiler types
        self.typ != CompilerType::Function && self.depth == 0
    }

    fn write_instr_const(
        &self,
        (u8_instr, u16_instr): (Instruction, Instruction),
        token: Rc<Token>,
        value: Value,
    ) -> Result<(), ()> {
        self.chunk
            .write_instr_const((u8_instr, u16_instr), Rc::clone(&token), value)
            .map_err(|_| self.err(TooManyConsts(token)))
    }

    fn write_const(&self, token: Rc<Token>, value: Value) -> Result<(), ()> {
        self.chunk
            .write_instr_const((CONSTANT8, CONSTANT16), token, value)
    }

    fn write_string_of_ident(&self, token: Rc<Token>) -> Result<(), ()> {
        self.write_const(Rc::clone(&token), Value::from(token.lexeme.clone()))
    }

    fn write_build(&self, instr: Instruction, token: Rc<Token>, size: usize) -> Result<(), ()> {
        self.chunk
            .write_build(instr, Rc::clone(&token), size)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn write_jump(
        &self,
        instr: Instruction,
        token: Rc<Token>,
    ) -> Box<dyn FnOnce() -> Result<(), ()> + '_> {
        let end = self.chunk.write_jump(instr, Rc::clone(&token));
        Box::new(move || end().map_err(|_| self.err(HugeJump(token))))
    }

    fn write_list_unpack(&self, token: Rc<Token>, to: usize) -> Result<(), ()> {
        self.chunk
            .write_list_unpack(Rc::clone(&token), to)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn write_hash_map_unpack(&self, token: Rc<Token>, defaults: Vec<bool>) -> Result<(), ()> {
        self.chunk
            .write_hash_map_unpack(Rc::clone(&token), defaults)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn quoted_string(&self, token: Rc<Token>) -> Result<String, ()> {
        let mut content = String::new();
        let mut iter = token.lexeme.chars().skip(1);
        while let Some(ch) = iter.next() {
            if ch == '\\' {
                if let Some(ch) = iter.next() {
                    match ch {
                        'n' => content.push('\n'),
                        'r' => content.push('\r'),
                        't' => content.push('\t'),
                        '\\' => content.push('\\'),
                        '"' => content.push('"'),
                        _ => {
                            self.err(BackSlashMisuse(token));
                            return Err(());
                        }
                    }
                } else {
                    self.err(BackSlashMisuse(token));
                    return Err(());
                }
            } else if ch == '"' {
                break;
            } else {
                content.push(ch);
            }
        }
        Ok(content)
    }

    fn unary(&self, op: &Rc<Token>, expr: &Expr) -> Result<(), ()> {
        self.expr(expr)?;
        match op.typ {
            TokenType::Minus => {
                self.chunk.write_instr_no_operands(NEGATE, Rc::clone(op));
            }
            TokenType::Bang => {
                self.chunk.write_instr_no_operands(NOT, Rc::clone(op));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn binary(&self, op: &Rc<Token>, lhs: &Expr, rhs: &Expr) -> Result<(), ()> {
        match op.typ {
            TokenType::Equal => {
                self.expr(rhs)?;
                self.des(DesType::Set, lhs)?;
                return Ok(());
            }
            _ => {}
        }

        self.expr(lhs)?;
        match op.typ {
            TokenType::And => {
                let end = self.write_jump(JUMP_IF_FALSE_OR_POP, Rc::clone(op));
                self.expr(rhs)?;
                end()?;
                return Ok(());
            }
            TokenType::Or => {
                let end = self.write_jump(JUMP_IF_TRUE_OR_POP, Rc::clone(op));
                self.expr(rhs)?;
                end()?;
                return Ok(());
            }
            _ => {}
        }
        self.expr(rhs)?;
        match op.typ {
            TokenType::Plus => self.chunk.write_instr_no_operands(ADD, Rc::clone(op)),
            TokenType::Minus => self.chunk.write_instr_no_operands(SUBTRACT, Rc::clone(op)),
            TokenType::Star => self.chunk.write_instr_no_operands(MULTIPLY, Rc::clone(op)),
            TokenType::Slash => self.chunk.write_instr_no_operands(DIVIDE, Rc::clone(op)),
            TokenType::Percent => self.chunk.write_instr_no_operands(REMAINDER, Rc::clone(op)),
            TokenType::DEqual => self.chunk.write_instr_no_operands(EQUAL, Rc::clone(op)),
            TokenType::BangEqual => self.chunk.write_instr_no_operands(NOT_EQUAL, Rc::clone(op)),
            TokenType::Greater => self.chunk.write_instr_no_operands(GREATER, Rc::clone(op)),
            TokenType::GreaterEqual => self
                .chunk
                .write_instr_no_operands(GREATER_EQUAL, Rc::clone(op)),
            TokenType::Less => self.chunk.write_instr_no_operands(LESS, Rc::clone(op)),
            TokenType::LessEqual => self
                .chunk
                .write_instr_no_operands(LESS_EQUAL, Rc::clone(op)),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn lambda(
        &self,
        token: &Rc<Token>,
        required: &Vec<Expr>,
        optional: &Vec<(Expr, Expr)>,
        variadic: &Option<(Rc<Token>, Box<Expr>)>,
        body: &Stml,
    ) -> Result<(), ()> {
        Ok(())
    }

    fn literal(&self, literal: &Literal) -> Result<(), ()> {
        match literal {
            Literal::Number(token) => {
                self.write_const(
                    Rc::clone(token),
                    Value::Number(token.lexeme.clone().parse().unwrap()),
                )?;
            }
            Literal::Bool(token) => {
                self.write_const(
                    Rc::clone(token),
                    Value::from(match token.typ {
                        TokenType::True => true,
                        TokenType::False => false,
                        _ => unreachable!(),
                    }),
                )?;
            }
            Literal::String(token) => {
                let value = Value::from(self.quoted_string(Rc::clone(token))?);
                self.write_const(Rc::clone(token), value)?;
            }
            Literal::Nil(token) => {
                self.write_const(Rc::clone(token), Value::Nil)?;
            }
            // TODO report HugeSize with better tokens
            Literal::List(token, exprs) => {
                let mut size = 0;
                for expr in exprs {
                    self.expr(expr)?;
                    size += 1;
                }
                self.chunk.write_build(BUILD_LIST, Rc::clone(token), size)?
            }
            Literal::Object(token, props) => {
                let mut size = 0;
                for (key, value, default) in props {
                    match value {
                        Some(lhs) => {
                            self.write_const(Rc::clone(key), Value::from(key.lexeme.clone()))?;
                            match default {
                                Some((op, rhs)) => self.binary(op, lhs, rhs)?,
                                None => self.expr(lhs)?,
                            }
                        }
                        None => match default {
                            Some((op, _)) => self.err(DefaultInObject(Rc::clone(op))),
                            None => {
                                // TODO get the value of the variable
                            }
                        },
                    }
                    size += 1;
                }
                self.write_build(BUILD_OBJECT, Rc::clone(token), size)?
            }
            Literal::Lambda(token, required, optional, variadic, body) => {
                self.lambda(token, required, optional, variadic, body)?
            }
        };
        Ok(())
    }

    fn resolve_local(&self, token: Rc<Token>) -> Option<usize> {
        self.locals.borrow().resolve_local(token)
    }

    fn resolve_upvalue(&self, token: Rc<Token>) -> Result<Option<usize>, ()> {
        self.locals.borrow_mut().resolve_upvalue(token)
    }

    fn get(&self, token: &Rc<Token>) -> Result<(), ()> {
        if let Some(idx) = self.resolve_local(Rc::clone(token)) {
            self.chunk
                .write_instr_idx(GET_LOCAL, Rc::clone(token), idx)
                .ok();
        } else {
            match self.resolve_upvalue(Rc::clone(token)) {
                Ok(idx) => match idx {
                    Some(idx) => {
                        self.chunk
                            .write_instr_idx(GET_UPVALUE, Rc::clone(token), idx)
                            .ok();
                    }
                    None => {
                        self.write_instr_const(
                            (GET_GLOBAL8, GET_GLOBAL16),
                            Rc::clone(token),
                            Value::from(token.lexeme.clone()),
                        )?;
                    }
                },
                Err(_) => {}
            }
        }
        Ok(())
    }

    fn expr(&self, expr: &Expr) -> Result<(), ()> {
        match expr {
            Expr::Variable(token) => self.get(token),
            Expr::Literal(literal) => self.literal(literal),
            Expr::Unary(op, expr) => self.unary(op, expr),
            Expr::Binary(op, lhs, rhs) => self.binary(op, lhs, rhs),
            _ => todo!(),
        }
    }

    fn define(&self, token: &Rc<Token>) -> Result<(), ()> {
        if self.global() {
            self.chunk.write_instr_const(
                (DEFINE_GLOBAL8, DEFINE_GLOBAL16),
                Rc::clone(token),
                Value::from(token.lexeme.clone()),
            )?
        } else {
            if let Some(idx) = self.locals.borrow().resolve_local(Rc::clone(token)) {
                if self.locals.borrow().get(idx).unwrap().1 == self.depth {
                    self.err(SameVarInScope(Rc::clone(token)))
                }
            }

            self.locals
                .borrow_mut()
                .push(Rc::clone(token))
                .map_err(|_| self.err(TooManyLocals(Rc::clone(token))))?;
            self.chunk
                .write_instr_no_operands(DEFINE_LOCAL, Rc::clone(token))
        }
        Ok(())
    }

    fn set(&self, token: &Rc<Token>) -> Result<(), ()> {
        if let Some(idx) = self.resolve_local(Rc::clone(token)) {
            self.chunk.write_instr_idx(SET_LOCAL, Rc::clone(token), idx)
        } else if let Some(idx) = self.resolve_upvalue(Rc::clone(token))? {
            self.chunk
                .write_instr_idx(SET_UPVALUE, Rc::clone(token), idx)
        } else {
            self.write_instr_const(
                (SET_GLOBAL8, SET_GLOBAL16),
                Rc::clone(token),
                Value::from(token.lexeme.clone()),
            )
        }
    }

    fn des(&self, typ: DesType, desable: &Expr) -> Result<(), ()> {
        macro_rules! oper {
            ($token:ident) => {
                match typ {
                    DesType::Define => self.define($token)?,
                    DesType::Set => self.set($token)?,
                }
            };
        }
        match desable {
            Expr::Variable(token) => oper!(token),
            Expr::Literal(Literal::List(token, exprs)) => {
                self.write_list_unpack(Rc::clone(token), exprs.len())?;
                for desable in exprs {
                    self.des(typ, desable)?
                }
            }
            Expr::Literal(Literal::Object(token, props)) => {
                // 1. Unpacking
                let mut defaults = vec![];
                for (key, value, default) in props {
                    self.write_string_of_ident(Rc::clone(key))?;
                    match value {
                        Some(expr) => match expr {
                            Expr::Binary(op, _, rhs) if op.typ == TokenType::Equal => {
                                self.expr(rhs)?;
                                defaults.push(true)
                            }
                            _ => defaults.push(false),
                        },
                        None => match default {
                            Some((_, expr)) => {
                                self.expr(expr)?;
                                defaults.push(true)
                            }
                            None => defaults.push(false),
                        },
                    }
                }
                self.write_hash_map_unpack(Rc::clone(token), defaults)?;
                // 2. Destructuring
                for (key, value, default) in props {
                    match value {
                        Some(expr) => match expr {
                            Expr::Binary(op, lhs, _) if op.typ == TokenType::Equal => {
                                self.des(typ, lhs)?
                            }
                            expr => self.des(typ, expr)?,
                        },
                        None => match default {
                            Some((_, _)) => {}
                            None => {
                                self.get(key)?;
                                oper!(token)
                            }
                        },
                    }
                }
            }
            expr => {
                self.err(InvalidDes(expr.token()));
                return Err(());
            }
        }
        Ok(())
    }

    fn var_decl(&self, _: &Rc<Token>, decls: &Vec<(Expr, Option<Expr>)>) -> Result<(), ()> {
        for (definable, init) in decls {
            match init {
                Some(expr) => self.expr(expr)?,
                None => {}
            }
            self.des(DesType::Define, definable)?
        }
        Ok(())
    }

    fn start_scope(&self) {
        self.locals.borrow_mut().depth += 1;
    }

    fn end_scope(&mut self) {
        // let locals = self.state.borrow().locals.clone();
        // let mut iter = locals.iter().rev();

        // while let Some(local) = iter.next() {
        //     if local.depth == self.scope_depth() {
        //         let local = self.state.borrow_mut().locals.pop().unwrap();
        //         if local.is_captured {
        //             self.chunk.write_instr(CloseUpValue, local.name);
        //         } else {
        //             self.chunk.write_instr(Pop, local.name);
        //         }
        //     } else {
        //         break;
        //     }
        // }

        // self.state.borrow_mut().scope_depth -= 1;
    }

    fn stml(&self, stml: &Stml) -> Result<(), ()> {
        match stml {
            Stml::VarDecl(token, decls) => self.var_decl(token, decls)?,
            Stml::Expr(expr) => {
                self.expr(expr)?;
                self.chunk
                    .write_instr_no_operands(POP, Rc::new(Token::default()))
            }
            Stml::Block(token, stmls) => {
                self.start_scope();
                self.stmls(stmls);
                self.end_scope();
            }
            _ => todo!(),
        }
        Ok(())
    }

    fn stmls(&self, stmls: &Vec<Stml>) {
        for stml in stmls {
            self.stml(stml).ok();
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, Vec<CompileError>> {
        self.stmls(self.ast);
        if self.errors.borrow().len() > 0 {
            Err(self.errors.borrow().clone())
        } else {
            Ok(self.chunk.clone())
        }
    }
}
