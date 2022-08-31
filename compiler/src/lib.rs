pub mod chunk;
pub mod error;

use chunk::value::{self, Arity, ArityType, Value};
use chunk::{Chunk, OpCode};
use error::CompileError;
use parser::ast::{Expr, Literal, Stml};
use parser::{token::*, Parser};
use std::path::{Path, PathBuf};
use std::{cell::RefCell, convert::From, fs, rc::Rc, string};

use CompileError::*;
use OpCode::*;
use TokenType::*;

#[derive(Debug, Clone)]
struct Local {
    token: Rc<Token>,
    depth: usize,
    captured: bool,
    exported: bool,
}

impl Local {
    fn new(token: Rc<Token>, depth: usize) -> Self {
        Self {
            token,
            depth,
            captured: false,
            exported: false,
        }
    }

    fn export(&mut self) {
        self.exported = true;
    }
}

#[derive(Debug, Clone)]
struct Locals {
    inner: Vec<Local>,
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

    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Fails when `self.inner` is larger than 256.
    fn push(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if self.inner.capacity() == self.inner.len() {
            Err(())
        } else {
            self.inner.push(Local::new(token, self.depth));
            Ok(())
        }
    }

    fn get(&self, idx: usize) -> &Local {
        &self.inner[idx]
    }

    fn pop(&mut self) -> Local {
        self.inner.pop().unwrap()
    }

    fn last_mut(&mut self) -> &mut Local {
        self.inner.last_mut().unwrap()
    }

    /// Fails when `self.upvalues` is larger than 256.
    fn add_upvalue(&mut self, local: bool, idx: usize) -> Result<usize, ()> {
        for (upvalue_idx, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.0 == local && upvalue.1 == idx {
                return Ok(upvalue_idx);
            }
        }
        if self.upvalues.capacity() == self.inner.len() {
            Err(())
        } else {
            let len = self.upvalues.len();
            self.upvalues.push((local, idx));
            Ok(len)
        }
    }

    /// `token` must be of type `Identifier`.
    fn resolve_upvalue(&mut self, token: Rc<Token>) -> Result<Option<usize>, ()> {
        match self.enclosing.clone() {
            Some(enclosing) => {
                let mut enclosing = enclosing.borrow_mut();
                if let Some(idx) = enclosing.resolve_local(Rc::clone(&token)) {
                    Ok(Some(self.add_upvalue(true, idx)?))
                } else {
                    enclosing.resolve_upvalue(token)
                }
            }
            None => Ok(None),
        }
    }

    /// `token` must be of type `Identifier`.
    fn resolve_local(&self, token: Rc<Token>) -> Option<usize> {
        for (
            idx,
            Local {
                token: local_token, ..
            },
        ) in self.inner.iter().enumerate().rev()
        {
            if token.lexeme() == local_token.lexeme() {
                return Some(idx);
            }
        }
        None
    }

    fn start_scope(&mut self) {
        self.depth += 1;
    }

    /// The returned vector represents whether the locals popped were captured or not.
    fn end_scope(&mut self) -> Vec<bool> {
        let mut tmp = vec![];
        self.depth -= 1;
        while let Some(Local { depth, .. }) = self.inner.last() {
            if *depth > self.depth {
                tmp.push(self.pop().captured);
            } else {
                break;
            }
        }
        tmp
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
    /// It must be of type `Function` for a function compiler and `EOF` for the rest.
    token: Rc<Token>,
    chunk: Chunk,
    locals: Rc<RefCell<Locals>>,
    /// A vector containing jumps ips.
    breaks: Vec<usize>,
    /// A vector containing enclosing loops starts.
    loops: Vec<usize>,
    errors: Vec<CompileError>,
}

impl<'a> Compiler<'a> {
    pub fn new(typ: CompilerType, ast: &'a Vec<Stml>, token: Rc<Token>) -> Self {
        Self {
            typ,
            ast,
            token,
            chunk: Chunk::new(),
            locals: Rc::new(RefCell::new(Locals::new(None))),
            breaks: vec![],
            loops: vec![],
            errors: vec![],
        }
    }

    fn new_function(token: Rc<Token>, body: &'a Stml, enclosing: Rc<RefCell<Locals>>) -> Self {
        let ast = match body {
            Stml::Block(_, stmls) => stmls,
            _ => unreachable!(),
        };
        Self {
            typ: CompilerType::Function,
            ast,
            token,
            chunk: Chunk::new(),
            locals: Rc::new(RefCell::new(Locals::new(Some(enclosing)))),
            breaks: vec![],
            loops: vec![],
            errors: vec![],
        }
    }

    fn err(&mut self, err: CompileError) {
        self.errors.push(err)
    }

    fn in_global(&self) -> bool {
        self.typ == CompilerType::Script && self.locals.borrow().depth == 0
    }

    fn in_loop(&self) -> bool {
        !self.loops.is_empty()
    }

    fn ip(&self) -> usize {
        self.chunk.len()
    }

    fn write_instr_const(
        &mut self,
        (u8_instr, u16_instr): (OpCode, OpCode),
        token: Rc<Token>,
        value: Value,
    ) -> Result<(), ()> {
        self.chunk
            .write_instr_const((u8_instr, u16_instr), Rc::clone(&token), value)
            .map_err(|_| self.err(TooManyConsts(token)))
    }

    fn write_const(&mut self, token: Rc<Token>, value: Value) -> Result<(), ()> {
        self.chunk
            .write_instr_const((CONST8, CONST16), token, value)
    }

    #[allow(unused_must_use)]
    fn nil(&mut self, token: Rc<Token>) {
        self.write_const(token, Value::Nil);
    }

    #[allow(unused_must_use)]
    fn bool(&mut self, token: Rc<Token>, value: bool) {
        self.write_const(token, Value::from(value));
    }

    #[allow(unused_must_use)]
    fn write_instr_idx(&mut self, op_code: OpCode, token: Rc<Token>, idx: usize) {
        self.chunk.write_instr_idx(op_code, token, idx);
    }

    fn write_string_of_ident(&mut self, token: Rc<Token>) -> Result<(), ()> {
        self.write_const(Rc::clone(&token), Value::from(token.lexeme().clone()))
    }

    fn write_build(&mut self, op_code: OpCode, token: Rc<Token>, size: usize) -> Result<(), ()> {
        self.chunk
            .write_build(op_code, Rc::clone(&token), size)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn settle_jump(&mut self, ip: usize) -> Result<(), ()> {
        self.chunk
            .settle_jump(ip)
            .map_err(|_| self.err(HugeJump(self.chunk.token(ip))))
    }

    fn write_loop(&mut self, token: Rc<Token>, ip: usize) -> Result<(), ()> {
        self.chunk
            .write_loop(Rc::clone(&token), ip)
            .map_err(|_| self.err(HugeJump(token)))
    }

    fn write_list_unpack(&mut self, token: Rc<Token>, to: usize) -> Result<(), ()> {
        self.chunk
            .write_list_unpack(Rc::clone(&token), to)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn write_hash_map_unpack(&mut self, token: Rc<Token>, defaults: Vec<bool>) -> Result<(), ()> {
        self.chunk
            .write_hash_map_unpack(Rc::clone(&token), defaults)
            .map_err(|_| self.err(HugeSize(token)))
    }

    fn write_closure(
        &mut self,
        token: Rc<Token>,
        function: value::Function,
        upvalues: Vec<(bool, usize)>,
    ) -> Result<(), ()> {
        self.chunk
            .write_closure(Rc::clone(&token), function, upvalues)
            .map_err(|_| self.err(TooManyConsts(token)))
    }

    fn write_call(&mut self, token: Rc<Token>, argc: usize) -> Result<(), ()> {
        self.chunk
            .write_call(Rc::clone(&token), argc)
            .map_err(|_| self.err(TooManyArgs(token)))
    }

    #[allow(unused_must_use)]
    fn write_call_unchecked(&mut self, token: Rc<Token>, argc: usize) {
        self.chunk.write_call(token, argc);
    }

    fn push(&mut self, token: Rc<Token>) -> Result<(), ()> {
        let mut locals = self.locals.borrow_mut();
        let res = locals.push(Rc::clone(&token));
        drop(locals);
        match res {
            Ok(_) => Ok(()),
            Err(_) => {
                self.err(TooManyLocals(token));
                Err(())
            }
        }
    }

    fn quoted_string(&mut self, token: Rc<Token>) -> Result<string::String, ()> {
        let mut content = string::String::new();
        let mut iter = token.lexeme().chars().skip(1);
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

    /// Parses quoted strings and unquoted ones.
    fn string(&mut self, token: Rc<Token>) -> Result<string::String, ()> {
        if token.lexeme().starts_with("\"") {
            self.quoted_string(token)
        } else {
            Ok(token.lexeme().clone())
        }
    }

    fn unary(&mut self, op: Rc<Token>, expr: &Expr) -> Result<(), ()> {
        self.expr(expr)?;
        match op.typ() {
            Minus => {
                self.chunk.write_instr_no_operands(NEG, op);
            }
            Bang => {
                self.chunk.write_instr_no_operands(NOT, op);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn binary(&mut self, lhs: &Expr, op: Rc<Token>, rhs: &Expr) -> Result<(), ()> {
        match op.typ() {
            Equal => {
                self.expr(rhs)?;
                self.chunk.write_instr_no_operands(DUP, op);
                self.settable(lhs)?;
                return Ok(());
            }
            PlusEqual | MinusEqual | StarEqual | SlashEqual | PercentEqual => match lhs {
                Expr::Variable(..) | Expr::Member(..) => {
                    self.get(lhs)?;
                    self.expr(rhs)?;
                    self.chunk.write_instr_no_operands(
                        match op.typ() {
                            PlusEqual => ADD,
                            MinusEqual => SUB,
                            StarEqual => MUL,
                            SlashEqual => DIV,
                            PercentEqual => REM,
                            _ => unreachable!(),
                        },
                        op,
                    );
                    self.set(lhs, false)?;
                    return Ok(());
                }
                _ => unreachable!(),
            },
            _ => {}
        }
        self.expr(lhs)?;
        match op.typ() {
            And => {
                let falsy_lhs = self.chunk.write_jump(JUMP_IF_FALSY_OR_POP, op);
                self.expr(rhs)?;
                self.settle_jump(falsy_lhs)?;
                return Ok(());
            }
            Or => {
                let truthy_lhs = self.chunk.write_jump(JUMP_IF_TRUTHY_OR_POP, op);
                self.expr(rhs)?;
                self.settle_jump(truthy_lhs)?;
                return Ok(());
            }
            _ => {}
        }
        self.expr(rhs)?;
        match op.typ() {
            Plus => self.chunk.write_instr_no_operands(ADD, op),
            Minus => self.chunk.write_instr_no_operands(SUB, op),
            Star => self.chunk.write_instr_no_operands(MUL, op),
            Slash => self.chunk.write_instr_no_operands(DIV, op),
            Percent => self.chunk.write_instr_no_operands(REM, op),
            DEqual => self.chunk.write_instr_no_operands(EQ, op),
            BangEqual => self.chunk.write_instr_no_operands(NOT_EQ, op),
            Greater => self.chunk.write_instr_no_operands(GREATER, op),
            GreaterEqual => self.chunk.write_instr_no_operands(GREATER_EQ, op),
            Less => self.chunk.write_instr_no_operands(LESS, op),
            LessEqual => self.chunk.write_instr_no_operands(LESS_EQ, op),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn lambda(
        &mut self,
        token: &Rc<Token>,
        required: &Vec<Expr>,
        optional: &Vec<(Expr, Expr)>,
        variadic: &Option<(Rc<Token>, Box<Expr>)>,
        body: &Stml,
    ) -> Result<(), ()> {
        self.function(body, required, optional, variadic, None, Rc::clone(token))
    }

    fn literal(&mut self, literal: &Literal) -> Result<(), ()> {
        match literal {
            Literal::Number(token) => {
                self.write_const(
                    Rc::clone(token),
                    Value::Number(token.lexeme().clone().parse().unwrap()),
                )?;
            }
            Literal::Bool(token) => {
                self.bool(
                    Rc::clone(token),
                    match token.typ() {
                        True => true,
                        False => false,
                        _ => unreachable!(),
                    },
                );
            }
            Literal::String(token) => {
                let value = Value::from(self.string(Rc::clone(token))?);
                self.write_const(Rc::clone(token), value)?;
            }
            Literal::Nil(token) => {
                self.nil(Rc::clone(token));
            }
            // TODO report HugeSize with better tokens
            Literal::List(token, exprs) => {
                let mut size = 0;
                for expr in exprs {
                    self.expr(expr)?;
                    size += 1;
                }
                self.write_build(BUILD_LIST, Rc::clone(token), size)?
            }
            Literal::Object(token, props) => {
                let mut size = 0;
                for (key, value, default) in props {
                    self.write_const(Rc::clone(key), Value::from(key.lexeme().clone()))?;
                    match value {
                        Some(lhs) => match default {
                            Some((op, rhs)) => self.binary(lhs, Rc::clone(op), rhs)?,
                            None => self.expr(lhs)?,
                        },
                        None => match default {
                            Some((op, _)) => self.err(DefaultInObject(Rc::clone(op))),
                            None => self.get(&Expr::Variable(Rc::clone(key)))?,
                        },
                    }
                    size += 1;
                }
                self.write_build(BUILD_HASH_MAP, Rc::clone(token), size)?
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

    /// `expr` must be a variable or member expressions, otherwise it panics.
    fn get(&mut self, expr: &Expr) -> Result<(), ()> {
        match expr {
            Expr::Variable(token) => {
                if let Some(idx) = self.resolve_local(Rc::clone(token)) {
                    self.write_instr_idx(GET_LOCAL, Rc::clone(token), idx);
                } else {
                    match self.resolve_upvalue(Rc::clone(token)) {
                        Ok(idx) => match idx {
                            Some(idx) => {
                                self.write_instr_idx(GET_UPVALUE, Rc::clone(token), idx);
                            }
                            None => {
                                self.write_instr_const(
                                    (GET_GLOBAL8, GET_GLOBAL16),
                                    Rc::clone(token),
                                    Value::from(token.lexeme().clone()),
                                )?;
                            }
                        },
                        Err(_) => {}
                    }
                }
            }
            Expr::Member(expr, op, key) => {
                self.expr(expr)?;
                self.expr(key)?;
                self.chunk.write_instr_no_operands(GET, Rc::clone(op));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn call(&mut self, callee: &Expr, op: Rc<Token>, exprs: &Vec<Expr>) -> Result<(), ()> {
        self.expr(callee)?;
        for arg in exprs {
            self.expr(arg)?
        }
        self.write_call(op, exprs.len())?;
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), ()> {
        match expr {
            Expr::Variable(..) | Expr::Member(..) => self.get(expr),
            Expr::Literal(literal) => self.literal(literal),
            Expr::Unary(op, expr) => self.unary(Rc::clone(op), expr),
            Expr::Binary(lhs, op, rhs) => self.binary(lhs, Rc::clone(op), rhs),
            Expr::Call(callee, op, exprs) => self.call(callee, Rc::clone(op), exprs),
        }
    }

    fn define(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if self.in_global() {
            self.chunk.write_instr_const(
                (DEF_GLOBAL8, DEF_GLOBAL16),
                Rc::clone(&token),
                Value::from(token.lexeme().clone()),
            )?
        } else {
            if token.lexeme().as_str() != "_" {
                if let Some(idx) = self.resolve_local(Rc::clone(&token)) {
                    if self.locals.borrow().get(idx).depth == self.locals.borrow().depth {
                        self.err(SameVarInScope(token));
                        return Err(());
                    }
                }
            }

            self.push(Rc::clone(&token))?;
            self.chunk.write_instr_no_operands(DEF_LOCAL, token)
        }
        Ok(())
    }

    fn can_export(&self) -> bool {
        self.typ != CompilerType::Function && self.locals.borrow().depth == 0
    }

    fn export(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if !self.can_export() {
            return Err(());
        }
        if !self.in_global() {
            self.define(Rc::clone(&token))?;
            self.locals.borrow_mut().last_mut().export();
        }
        Ok(())
    }

    fn unpack_hash_map(
        &mut self,
        token: Rc<Token>,
        props: &Vec<(Rc<Token>, Option<Expr>, Option<(Rc<Token>, Expr)>)>,
    ) -> Result<(), ()> {
        let mut defaults = vec![];
        for (key, _, default) in props {
            self.write_string_of_ident(Rc::clone(key))?;
            match default {
                Some((_, expr)) => {
                    self.expr(expr)?;
                    defaults.push(true)
                }
                None => defaults.push(false),
            }
        }
        self.write_hash_map_unpack(token, defaults)
    }

    /// `expr` must be a variable or member expressions, otherwise it panics.
    fn set(&mut self, expr: &Expr, pop: bool) -> Result<(), ()> {
        match expr {
            Expr::Variable(token) => {
                if let Some(idx) = self.resolve_local(Rc::clone(token)) {
                    self.chunk
                        .write_instr_idx(SET_LOCAL, Rc::clone(token), idx)?
                } else if let Some(idx) = self.resolve_upvalue(Rc::clone(token))? {
                    self.chunk
                        .write_instr_idx(SET_UPVALUE, Rc::clone(token), idx)?
                } else {
                    self.write_instr_const(
                        (SET_GLOBAL8, SET_GLOBAL16),
                        Rc::clone(token),
                        Value::from(token.lexeme().clone()),
                    )?
                }
            }
            Expr::Member(expr, op, key) => {
                self.expr(expr)?;
                self.expr(key)?;
                self.chunk.write_instr_no_operands(SET, Rc::clone(op))
            }
            _ => unreachable!(),
        }
        if pop {
            self.chunk.write_instr_no_operands(POP, expr.token());
        }
        Ok(())
    }

    fn settable(&mut self, settable: &Expr) -> Result<(), ()> {
        match settable {
            Expr::Variable(..) | Expr::Member(..) => self.set(settable, true)?,
            Expr::Literal(Literal::List(token, exprs)) => {
                self.write_list_unpack(Rc::clone(token), exprs.len())?;
                for settable in exprs.iter().rev() {
                    self.settable(settable)?
                }
            }
            Expr::Literal(Literal::Object(token, props)) => {
                // 1. Unpacking
                self.unpack_hash_map(Rc::clone(token), props)?;
                // 2. Destructuring
                for (key, value, _) in props {
                    match value {
                        Some(expr) => self.settable(expr)?,
                        None => self.set(&Expr::Variable(Rc::clone(key)), true)?,
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

    fn definable(&mut self, definable: &Expr, export: bool) -> Result<(), ()> {
        macro_rules! oper {
            ($token:ident) => {
                if export {
                    self.export(Rc::clone($token))?
                } else {
                    self.define(Rc::clone($token))?
                }
            };
        }

        match definable {
            Expr::Variable(token) => oper!(token),
            Expr::Literal(Literal::List(token, exprs)) => {
                self.write_list_unpack(Rc::clone(token), exprs.len())?;
                for definable in exprs.iter().rev() {
                    self.definable(definable, export)?
                }
            }
            Expr::Literal(Literal::Object(token, props)) => {
                // 1. Unpacking
                self.unpack_hash_map(Rc::clone(token), props)?;
                // 2. Destructuring
                for (key, value, _) in props {
                    match value {
                        Some(expr) => self.definable(expr, export)?,
                        None => {
                            oper!(key)
                        }
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

    fn var_decl(
        &mut self,
        export_token: &Option<Rc<Token>>,
        token: Rc<Token>,
        decls: &Vec<(Expr, Option<Expr>)>,
    ) -> Result<(), ()> {
        for (definable, init) in decls {
            match init {
                Some(expr) => self.expr(expr)?,
                None => self.nil(Rc::clone(&token)),
            }
            self.definable(definable, export_token.is_some())?
        }
        Ok(())
    }

    fn start_scope(&self) {
        self.locals.borrow_mut().start_scope();
    }

    fn end_scope(&mut self, token: Rc<Token>) {
        for captured in self.locals.borrow_mut().end_scope() {
            self.chunk.write_instr_no_operands(
                if captured { CLOSE_UPVALUE } else { POP_LOCAL },
                Rc::clone(&token),
            )
        }
    }

    fn if_stml(
        &mut self,
        token: &Rc<Token>,
        condition: &Expr,
        body: &Box<Stml>,
        elseifs: &Vec<(Rc<Token>, Expr, Stml)>,
        else_: &Option<(Rc<Token>, Box<Stml>)>,
    ) -> Result<(), ()> {
        self.expr(condition)?;
        let falsy_condition = self.chunk.write_jump(POP_JUMP_IF_FALSY, Rc::clone(token));
        self.stml(body)?;
        let mut end = vec![self.chunk.write_jump(JUMP, body.token())];
        self.settle_jump(falsy_condition)?;
        for (token, condition, body) in elseifs {
            self.expr(condition)?;
            let falsy_condition = self.chunk.write_jump(POP_JUMP_IF_FALSY, Rc::clone(token));
            self.stml(body)?;
            end.push(self.chunk.write_jump(JUMP, Rc::clone(token)));
            self.settle_jump(falsy_condition)?;
        }
        match else_ {
            Some((_, body)) => {
                self.stml(body)?;
            }
            None => {}
        }
        for jump in end {
            self.settle_jump(jump)?;
        }
        Ok(())
    }

    fn params(
        &mut self,
        required: &Vec<Expr>,
        optional: &Vec<(Expr, Expr)>,
        variadic: &Option<(Rc<Token>, Box<Expr>)>,
    ) -> Result<(Arity, Vec<usize>, usize), ()> {
        let mut defaults = vec![];
        for (_, default) in optional {
            defaults.push(self.ip());
            self.expr(default)?
        }
        let body = self.ip();
        if let Some((token, definable)) = variadic {
            self.chunk
                .write_instr_no_operands(BUILD_VARIADIC, Rc::clone(token));
            self.definable(definable, false)?
        }
        for (definable, _) in optional.iter().rev() {
            self.definable(definable, false)?
        }
        for definable in required.iter().rev() {
            self.definable(definable, false)?
        }
        Ok((
            Arity::new(
                if variadic.is_some() {
                    ArityType::Variadic
                } else {
                    ArityType::Fixed
                },
                required.len(),
                optional.len(),
            ),
            defaults,
            body,
        ))
    }

    fn function(
        &mut self,
        body: &Stml,
        required: &Vec<Expr>,
        optional: &Vec<(Expr, Expr)>,
        variadic: &Option<(Rc<Token>, Box<Expr>)>,
        name: Option<Rc<Token>>,
        token: Rc<Token>,
    ) -> Result<(), ()> {
        let mut compiler = Compiler::new_function(Rc::clone(&token), body, Rc::clone(&self.locals));
        let (arity, defaults, body) = compiler.params(required, optional, variadic)?;
        if let Some(token) = &name {
            compiler.define(Rc::clone(token))?
        } else {
            compiler
                .chunk
                .write_instr_no_operands(POP, Rc::clone(&token))
        };
        let chunk = match compiler.compile() {
            Ok(chunk) => chunk,
            Err(errors) => {
                for err in errors {
                    self.err(err);
                }
                return Err(());
            }
        };
        let upvalues = compiler.locals.borrow().upvalues.clone();
        self.write_closure(
            token,
            value::Function::new(
                name.map(|token| token.lexeme().clone()),
                chunk,
                arity,
                defaults,
                body,
            ),
            upvalues,
        )?;
        Ok(())
    }

    fn function_decl(
        &mut self,
        export_token: &Option<Rc<Token>>,
        token: Rc<Token>,
        name: Rc<Token>,
        required: &Vec<Expr>,
        optional: &Vec<(Expr, Expr)>,
        variadic: &Option<(Rc<Token>, Box<Expr>)>,
        body: &Box<Stml>,
    ) -> Result<(), ()> {
        self.function(
            body,
            required,
            optional,
            variadic,
            Some(Rc::clone(&name)),
            token,
        )?;
        match export_token {
            Some(_) => self.export(name)?,
            None => self.define(name)?,
        }
        Ok(())
    }

    fn return_stml(&mut self, token: Rc<Token>, value: &Option<Expr>) -> Result<(), ()> {
        if self.typ != CompilerType::Function {
            self.err(ReturnOutsideFunction(Rc::clone(&token)));
            return Err(());
        }
        match value {
            Some(expr) => self.expr(expr)?,
            None => self.nil(Rc::clone(&token)),
        };
        self.chunk.write_instr_no_operands(RET, token);
        Ok(())
    }

    fn throw_stml(&mut self, token: Rc<Token>, value: &Option<Expr>) -> Result<(), ()> {
        match value {
            Some(expr) => self.expr(expr)?,
            None => self.nil(Rc::clone(&token)),
        };
        self.chunk.write_instr_no_operands(THROW, token);
        Ok(())
    }

    fn settle_breaks(&mut self) -> Result<(), ()> {
        while let Some(ip) = self.breaks.pop() {
            self.settle_jump(ip)?
        }
        Ok(())
    }

    fn loop_stml(&mut self, token: Rc<Token>, body: &Stml) -> Result<(), ()> {
        let start = self.ip();
        self.loops.push(start);
        self.stml(body)?;
        self.write_loop(token, start)?;
        self.settle_breaks()
    }
    fn while_stml(&mut self, token: Rc<Token>, condition: &Expr, body: &Stml) -> Result<(), ()> {
        let start = self.ip();
        self.loops.push(start);
        self.expr(condition)?;
        let falsy_condition = self.chunk.write_jump(POP_JUMP_IF_FALSY, Rc::clone(&token));
        self.stml(body)?;
        self.write_loop(token, start)?;
        self.settle_jump(falsy_condition)?;
        self.settle_breaks()
    }

    fn for_in_stml(
        &mut self,
        token: Rc<Token>,
        definable: &Expr,
        _: Rc<Token>,
        iterable: &Expr,
        body: &Stml,
    ) -> Result<(), ()> {
        self.expr(iterable)?;
        self.chunk.write_instr_no_operands(ITER, Rc::clone(&token));
        let start = self.ip();
        self.loops.push(start);
        let iterator_stopped = self.chunk.write_jump(FOR_ITER, token);
        match body {
            Stml::Block(token, stmls) => {
                self.start_scope();
                self.definable(definable, false)?;
                self.stmls(stmls);
                self.end_scope(Rc::clone(token));
                self.write_loop(Rc::clone(token), start)?
            }
            _ => unreachable!(),
        }
        self.settle_jump(iterator_stopped)?;
        self.settle_breaks()
    }

    fn break_stml(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if !self.in_loop() {
            self.err(OutsideLoopBreak(token));
            return Err(());
        }
        self.breaks.push(self.ip());
        Ok(())
    }

    fn continue_stml(&mut self, token: Rc<Token>) -> Result<(), ()> {
        if !self.in_loop() {
            self.err(OutsideLoopContinue(token));
            return Err(());
        }
        self.write_loop(token, *self.loops.last().unwrap())
    }

    fn try_catch_stml(
        &mut self,
        token: Rc<Token>,
        body: &Stml,
        _: Rc<Token>,
        err: Rc<Token>,
        catch_body: &Stml,
    ) -> Result<(), ()> {
        let caught = self.chunk.write_jump(APPEND_HANDLER, token);
        self.stml(body)?;
        self.settle_jump(caught)?;
        match catch_body {
            Stml::Block(token, stmls) => {
                self.start_scope();
                self.define(err)?;
                self.stmls(stmls);
                self.end_scope(Rc::clone(token))
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn import_stml(
        &mut self,
        token: Rc<Token>,
        definable: &Expr,
        _: Rc<Token>,
        path: Rc<Token>,
    ) -> Result<(), ()> {
        if !self.can_export() {
            self.err(InvalidImportUsage(token));
            return Err(());
        }
        let path = {
            let tmp = self.quoted_string(path)?;
            match token.path() {
                Some(path) => path.parent().unwrap_or(&Path::new("")).join(tmp),
                None => PathBuf::from(tmp),
            }
        };
        let source = fs::read_to_string(&path)
            .map_err(|err| self.err(Io(Rc::clone(&token), Rc::new(err))))?;
        let (ast, token) = Parser::new(source, Some(path))
            .parse()
            .map_err(|errors| self.err(ModuleParser(Rc::clone(&token), errors)))?;
        let chunk = Compiler::new(CompilerType::Module, &ast, Rc::clone(&token))
            .compile()
            .map_err(|errors| {
                for err in errors {
                    self.err(err)
                }
            })?;
        self.write_closure(
            Rc::clone(&token),
            value::Function::new(None, chunk, Arity::default(), vec![], 0),
            vec![],
        )?;
        self.write_call_unchecked(token, 0);
        self.definable(definable, false)?;
        Ok(())
    }

    fn stml(&mut self, stml: &Stml) -> Result<(), ()> {
        match stml {
            Stml::VarDecl(export_token, token, decls) => {
                self.var_decl(export_token, Rc::clone(token), decls)?
            }
            Stml::FunctionDecl(export_token, token, name, required, optional, variadic, body) => {
                self.function_decl(
                    export_token,
                    Rc::clone(token),
                    Rc::clone(name),
                    required,
                    optional,
                    variadic,
                    body,
                )?
            }
            Stml::Expr(expr) => {
                self.expr(expr)?;
                self.chunk.write_instr_no_operands(POP, expr.token())
            }
            Stml::Block(token, stmls) => {
                self.start_scope();
                self.stmls(stmls);
                self.end_scope(Rc::clone(token));
            }
            Stml::If(token, condition, body, elseifs, else_) => {
                self.if_stml(token, condition, body, elseifs, else_)?
            }
            Stml::Return(token, value) => self.return_stml(Rc::clone(token), value)?,
            Stml::Throw(token, value) => self.throw_stml(Rc::clone(token), value)?,
            Stml::While(token, condition, body) => {
                self.while_stml(Rc::clone(token), condition, body)?
            }
            Stml::Loop(token, body) => self.loop_stml(Rc::clone(token), body)?,
            Stml::ForIn(token, definable, in_token, iterable, body) => self.for_in_stml(
                Rc::clone(token),
                definable,
                Rc::clone(in_token),
                iterable,
                body,
            )?,
            Stml::Break(token) => self.break_stml(Rc::clone(token))?,
            Stml::Continue(token) => self.continue_stml(Rc::clone(token))?,
            Stml::TryCatch(token, body, catch_token, err, catch_body) => self.try_catch_stml(
                Rc::clone(token),
                body,
                Rc::clone(catch_token),
                Rc::clone(err),
                catch_body,
            )?,
            Stml::Import(token, definable, from_token, path) => self.import_stml(
                Rc::clone(token),
                definable,
                Rc::clone(from_token),
                Rc::clone(path),
            )?,
        }
        Ok(())
    }

    #[allow(unused_must_use)]
    fn stmls(&mut self, stmls: &Vec<Stml>) {
        for stml in stmls {
            self.stml(stml); // ?
        }
    }

    #[allow(unused_must_use)]
    pub fn compile(&mut self) -> Result<Chunk, Vec<CompileError>> {
        if cfg!(feature = "verbose") && self.typ == CompilerType::Script {
            println!("[COMPILER] started")
        }
        self.stmls(self.ast);
        match self.typ {
            CompilerType::Script => {}
            CompilerType::Function => {
                self.write_const(Rc::clone(&self.token), Value::Nil);
                self.chunk
                    .write_instr_no_operands(RET, Rc::clone(&self.token));
            }
            CompilerType::Module => {
                let locals = self.locals.borrow().clone();
                let mut size = 0;
                for idx in 0..locals.len() {
                    let local = locals.get(idx);
                    if local.exported {
                        self.write_const(
                            Rc::clone(&local.token),
                            Value::from(local.token.lexeme().clone()),
                        );
                        self.write_instr_idx(GET_LOCAL, Rc::clone(&local.token), idx);
                        size += 1;
                    }
                }
                self.chunk
                    .write_build(BUILD_HASH_MAP, Rc::clone(&self.token), size)
                    .map_err(|_| TooManyExports(Rc::clone(&self.token))); // ?
                self.chunk
                    .write_instr_no_operands(RET, Rc::clone(&self.token))
            }
        }
        if self.errors.len() > 0 {
            if cfg!(feature = "verbose") && self.typ == CompilerType::Script {
                println!("[COMPILER] failed")
            }
            Err(self.errors.clone())
        } else {
            if cfg!(feature = "verbose") && self.typ == CompilerType::Script {
                println!("[COMPILER] succeeded");
                println!("{:?}", self.chunk)
            }
            Ok(self.chunk.clone())
        }
    }
}
