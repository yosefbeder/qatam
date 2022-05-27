use super::{
    ast::{Expr, Literal, Stml},
    chunk::{Chunk, Instruction},
    reporter::{Phase, Report, Reporter},
    token::{Token, TokenType},
    value::{Arity, Function, Value},
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CompilerType {
    Script,
    Function,
}

#[derive(Debug, Clone)]
struct Local<'a> {
    name: Rc<Token<'a>>,
    depth: u32,
    is_captured: bool,
}

#[derive(Debug, Clone)]
struct UpValue {
    is_local: bool,
    idx: usize,
}

impl UpValue {
    fn new(is_local: bool, idx: usize) -> Self {
        Self { is_local, idx }
    }
}

impl<'a> Local<'a> {
    fn new(name: Rc<Token<'a>>, depth: u32) -> Local<'a> {
        Local {
            name,
            depth,
            is_captured: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Loop {
    start: usize,
    breaks: Vec<usize>,
}

impl Loop {
    fn new(start: usize) -> Loop {
        Loop {
            start,
            breaks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerState<'a> {
    scope_depth: u32,
    locals: Vec<Local<'a>>, //TODO make it limited as the stack is
    up_values: Vec<UpValue>,
    had_error: bool,
    loops: Vec<Loop>,
    enclosing_state: Option<Rc<RefCell<CompilerState<'a>>>>,
}

impl<'a> CompilerState<'a> {
    fn new(enclosing_state: Option<Rc<RefCell<CompilerState<'a>>>>) -> Self {
        Self {
            scope_depth: 0,
            locals: Vec::new(),
            up_values: Vec::new(),
            had_error: false,
            loops: Vec::new(),
            enclosing_state,
        }
    }

    fn append_up_value(&mut self, is_local: bool, idx: usize) -> usize {
        for (i, up_value) in self.up_values.iter().enumerate() {
            if up_value.is_local == is_local && up_value.idx == idx {
                return i;
            }
        }
        let up_value_index = self.up_values.len();
        self.up_values.push(UpValue::new(is_local, idx));
        up_value_index
    }

    fn resolve_local(&self, token: Rc<Token<'a>>) -> Option<usize> {
        let mut iter = self.locals.iter().enumerate().rev();

        while let Some((idx, local)) = iter.next() {
            if local.name == token {
                return Some(idx);
            }
        }

        None
    }

    fn resolve_up_value(&mut self, token: Rc<Token<'a>>) -> Option<usize> {
        if self.enclosing_state.is_none() {
            return None;
        }

        let mut enclosing_state = self.enclosing_state.as_ref().unwrap().borrow_mut();
        match enclosing_state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                enclosing_state.locals.get_mut(idx).unwrap().is_captured = true;
                drop(enclosing_state);
                return Some(self.append_up_value(true, idx));
            }
            _ => {}
        }
        match enclosing_state.resolve_up_value(Rc::clone(&token)) {
            Some(idx) => {
                drop(enclosing_state);
                Some(self.append_up_value(false, idx))
            }
            _ => None,
        }
    }
}

pub struct Compiler<'a, 'b, 'c> {
    typ: CompilerType,
    name: Option<String>,
    arity: u8,
    ast: &'c Vec<Stml<'a>>,
    chunk: Chunk<'a>,
    reporter: &'b mut dyn Reporter<'a>,
    state: Rc<RefCell<CompilerState<'a>>>,
}

impl<'a, 'b, 'c> Compiler<'a, 'b, 'c> {
    pub fn new(ast: &'c Vec<Stml<'a>>, reporter: &'b mut dyn Reporter<'a>) -> Self {
        Compiler {
            typ: CompilerType::Script,
            name: None,
            arity: 0,
            ast,
            chunk: Chunk::new(),
            reporter,
            state: Rc::new(RefCell::new(CompilerState::new(None))),
        }
    }

    fn error_at(&mut self, token: Rc<Token<'a>>, msg: &str) {
        let report = Report::new(Phase::Compilation, msg.to_string(), token);
        self.reporter.error(report);
        self.state.borrow_mut().had_error = true;
    }

    // fn warning_at(&self, token: &Token<'b>, msg: &str) {
    //     let report = Report::new(Phase::Parsing, msg.to_string(), Rc::new(token.clone()));
    //     self.reporter.warning(report);
    // }

    fn in_global_scope(&self) -> bool {
        self.typ == CompilerType::Script && self.state.borrow().scope_depth == 0
    }

    fn in_function(&self) -> bool {
        self.typ == CompilerType::Function
    }

    fn new_function(
        name: Option<String>,
        body: &'c Stml<'a>,
        enclosing_state: Rc<RefCell<CompilerState<'a>>>,
        reporter: &'b mut dyn Reporter<'a>,
    ) -> Self {
        Compiler {
            typ: CompilerType::Function,
            name,
            arity: 0,
            ast: match body {
                Stml::Block(stmls) => stmls,
                _ => unreachable!(),
            },
            chunk: Chunk::new(),
            reporter,
            state: Rc::new(RefCell::new(CompilerState::new(Some(enclosing_state)))),
        }
    }

    fn define_variable(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        let scope_depth = self.state.borrow().scope_depth;

        if self.in_global_scope() {
            self.chunk
                .emit_const(Value::String(token.lexeme.clone()), Some(Rc::clone(&token)))?;
            self.chunk
                .emit_instr(Instruction::DefineGlobal, Some(Rc::clone(&token)));
            return Ok(());
        }

        let locals = &self.state.borrow().locals.clone();
        let mut iter = locals.iter().rev();
        while let Some(local) = iter.next() {
            if local.depth != scope_depth {
                break;
            }
            if local.name == token {
                self.error_at(
                    token,
                    "لا يمكنك تعريف نفس المتغير أكثر من مرة في نفس المجموعة",
                );
                return Err(());
            }
        }

        let local = Local::new(token, scope_depth);
        self.state.borrow_mut().locals.push(local);
        Ok(())
    }

    fn set_global(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        self.chunk
            .emit_const(Value::String(token.lexeme.clone()), Some(Rc::clone(&token)))?;
        self.chunk
            .emit_instr(Instruction::SetGlobal, Some(Rc::clone(&token)));
        return Ok(());
    }

    fn set_variable(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        if self.in_global_scope() {
            return self.set_global(Rc::clone(&token));
        }

        let state = self.state.borrow();
        match state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk
                    .emit_instr(Instruction::SetLocal, Some(Rc::clone(&token)));
                self.chunk.emit_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        let mut state = self.state.borrow_mut();
        match state.resolve_up_value(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk
                    .emit_instr(Instruction::SetUpValue, Some(Rc::clone(&token)));
                self.chunk.emit_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        self.set_global(Rc::clone(&token))
    }

    fn get_variable(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        let state = self.state.borrow();
        match state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk
                    .emit_instr(Instruction::GetLocal, Some(Rc::clone(&token)));
                self.chunk.emit_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        let mut state = self.state.borrow_mut();
        match state.resolve_up_value(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk
                    .emit_instr(Instruction::GetUpValue, Some(Rc::clone(&token)));
                self.chunk.emit_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        self.chunk
            .emit_const(Value::String(token.lexeme.clone()), Some(Rc::clone(&token)))?;
        self.chunk
            .emit_instr(Instruction::GetGlobal, Some(Rc::clone(&token)));
        Ok(())
    }

    fn literal(&mut self, literal: &Literal<'a>) -> Result<(), ()> {
        match literal {
            Literal::Number(token) => {
                self.chunk.emit_const(
                    Value::Number(token.lexeme.clone().parse().unwrap()),
                    Some(Rc::clone(token)),
                )?;
            }
            Literal::Bool(token) => {
                self.chunk.emit_const(
                    Value::Bool(match token.typ {
                        TokenType::True => true,
                        TokenType::False => false,
                        _ => unreachable!(),
                    }),
                    Some(Rc::clone(token)),
                )?;
            }
            Literal::String(token) => {
                let mut content = String::new();
                let mut iter = token.lexeme.chars();

                if let Some(c) = iter.next() {
                    if c != '"' {
                        content.push(c)
                    }
                }

                while let Some(c) = iter.next() {
                    if c == '\\' {
                        if let Some(c) = iter.next() {
                            match c {
                                'n' => content.push('\n'),
                                'r' => content.push('\r'),
                                't' => content.push('\t'),
                                '\\' => content.push('\\'),
                                '"' => content.push('"'),
                                '\'' => content.push('\''),
                                '0' => content.push('\0'),
                                _ => {
                                    //TODO add a hint here
                                    self.error_at(Rc::clone(&token), "رمز غير متوقع بعد '\\'");
                                    return Err(());
                                }
                            }
                        }
                    } else if c == '"' {
                        break;
                    } else {
                        content.push(c);
                    }
                }

                let value = Value::String(content);

                self.chunk.emit_const(value, Some(Rc::clone(token)))?;
            }
            Literal::Nil(token) => {
                self.chunk.emit_const(Value::Nil, Some(Rc::clone(token)))?;
            }
            Literal::List(exprs) => {
                let mut size = 0;
                for expr in exprs {
                    self.expr(expr)?;
                    size += 1;
                }
                self.chunk.emit_instr(Instruction::BuildList, None);
                self.chunk.emit_byte(size);
            }
            Literal::Object(items) => {
                let mut size = 0;
                for item in items {
                    self.chunk.emit_const(
                        Value::String(item.0.lexeme.clone()),
                        Some(Rc::clone(&item.0)),
                    )?;
                    self.expr(&item.1)?;
                    size += 1;
                }
                self.chunk.emit_instr(Instruction::BuildObject, None);
                self.chunk.emit_byte(size);
            }
        };
        Ok(())
    }

    fn unary(&mut self, op: Rc<Token<'a>>, expr: &Expr<'a>) -> Result<(), ()> {
        self.expr(expr)?;
        match op.typ {
            TokenType::Minus => {
                self.chunk
                    .emit_instr(Instruction::Negate, Some(Rc::clone(&op)));
            }
            TokenType::Bang => {
                self.chunk
                    .emit_instr(Instruction::Not, Some(Rc::clone(&op)));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn binary(&mut self, op: Rc<Token<'a>>, left: &Expr<'a>, right: &Expr<'a>) -> Result<(), ()> {
        if op.typ == TokenType::Equal {
            match left {
                Expr::Variable(token) => {
                    self.expr(right)?;
                    self.set_variable(Rc::clone(token))?;
                }
                _ => unreachable!(),
            }
            return Ok(());
        }

        self.expr(left)?;

        match op.typ {
            TokenType::And => {
                let false_jump = self
                    .chunk
                    .emit_jump(Instruction::JumpIfFalse, Some(Rc::clone(&op)));
                self.chunk
                    .emit_instr(Instruction::Pop, Some(Rc::clone(&op)));
                self.expr(right)?;
                self.chunk.patch_jump(false_jump);
                return Ok(());
            }
            TokenType::Or => {
                let true_jump = self
                    .chunk
                    .emit_jump(Instruction::JumpIfTrue, Some(Rc::clone(&op)));
                self.chunk
                    .emit_instr(Instruction::Pop, Some(Rc::clone(&op)));
                self.expr(right)?;
                self.chunk.patch_jump(true_jump);
                return Ok(());
            }
            _ => {}
        }

        self.expr(right)?;
        match op.typ {
            TokenType::Plus => {
                self.chunk
                    .emit_instr(Instruction::Add, Some(Rc::clone(&op)));
            }
            TokenType::Minus => {
                self.chunk
                    .emit_instr(Instruction::Subtract, Some(Rc::clone(&op)));
            }
            TokenType::Star => {
                self.chunk
                    .emit_instr(Instruction::Multiply, Some(Rc::clone(&op)));
            }
            TokenType::Slash => {
                self.chunk
                    .emit_instr(Instruction::Divide, Some(Rc::clone(&op)));
            }
            TokenType::Percent => {
                self.chunk
                    .emit_instr(Instruction::Remainder, Some(Rc::clone(&op)));
            }
            TokenType::DEqual => {
                self.chunk
                    .emit_instr(Instruction::Equal, Some(Rc::clone(&op)));
            }
            TokenType::BangEqual => {
                self.chunk
                    .emit_instr(Instruction::Equal, Some(Rc::clone(&op)));
                self.chunk
                    .emit_instr(Instruction::Not, Some(Rc::clone(&op)));
            }
            TokenType::Greater => {
                self.chunk
                    .emit_instr(Instruction::Greater, Some(Rc::clone(&op)));
            }
            TokenType::GreaterEqual => {
                self.chunk
                    .emit_instr(Instruction::GreaterEqual, Some(Rc::clone(&op)));
            }
            TokenType::Less => {
                self.chunk
                    .emit_instr(Instruction::Less, Some(Rc::clone(&op)));
            }
            TokenType::LessEqual => {
                self.chunk
                    .emit_instr(Instruction::LessEqual, Some(Rc::clone(&op)));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    //TODO check reassure that 'left' and 'right' works the way you want
    fn get(&mut self, token: Rc<Token<'a>>, instance: &Expr<'a>, key: &Expr<'a>) -> Result<(), ()> {
        self.expr(instance)?;
        self.expr(key)?;
        self.chunk
            .emit_instr(Instruction::Get, Some(Rc::clone(&token)));
        Ok(())
    }

    fn set(
        &mut self,
        token: Rc<Token<'a>>,
        instance: &Expr<'a>,
        key: &Expr<'a>,
        value: &Expr<'a>,
    ) -> Result<(), ()> {
        self.expr(value)?;
        self.expr(instance)?;
        self.expr(key)?;
        self.chunk
            .emit_instr(Instruction::Set, Some(Rc::clone(&token)));
        Ok(())
    }

    fn call(
        &mut self,
        token: Rc<Token<'a>>,
        callee: &Expr<'a>,
        args: &Vec<Expr<'a>>,
    ) -> Result<(), ()> {
        self.expr(callee)?;
        let mut count = 0;
        for arg in args {
            if count == 0xff {
                self.error_at(token, "عدد كثر من المدخلات");
                return Err(());
            }
            self.expr(arg)?;
            count += 1;
        }
        self.chunk
            .emit_instr(Instruction::Call, Some(Rc::clone(&token)));
        self.chunk.emit_byte(count as u8);
        Ok(())
    }

    pub fn expr(&mut self, expr: &Expr<'a>) -> Result<(), ()> {
        match expr {
            Expr::Variable(token) => self.get_variable(Rc::clone(token))?,
            Expr::Literal(literal) => self.literal(literal)?,
            Expr::Unary(op, expr) => self.unary(Rc::clone(op), expr)?,
            Expr::Binary(op, left, right) => self.binary(Rc::clone(op), left, right)?,
            Expr::Get(token, instance, key) => self.get(Rc::clone(&token), instance, key)?,
            Expr::Set(token, instance, key, value) => {
                self.set(Rc::clone(&token), instance, key, value)?
            }
            Expr::Call(token, callee, args) => self.call(Rc::clone(&token), callee, args)?,
        };
        Ok(())
    }

    fn define_params(&mut self, params: &Vec<Rc<Token<'a>>>) -> Result<(), ()> {
        if self.typ == CompilerType::Script {
            unreachable!();
        }

        for param in params {
            if self.arity == 0xff {
                self.error_at(Rc::clone(param), "عدد كثير من المعاملات");
                return Err(());
            }
            self.define_variable(Rc::clone(param))?;
            self.arity += 1;
        }

        Ok(())
    }

    fn function_decl(
        &mut self,
        name: Rc<Token<'a>>,
        params: &Vec<Rc<Token<'a>>>,
        body: &Stml<'a>,
    ) -> Result<(), ()> {
        let mut function_compiler = Compiler::new_function(
            Some(name.lexeme.clone()),
            body,
            Rc::clone(&self.state),
            self.reporter,
        );
        function_compiler.define_variable(Rc::clone(&name))?;
        function_compiler.define_params(params)?;
        self.chunk.emit_const(
            Value::Function(Rc::new(function_compiler.compile()?)),
            Some(Rc::clone(&name)),
        )?;
        //TODO consider not appending regular functions as closures optimization
        let up_values = &function_compiler.state.borrow().up_values;
        self.chunk
            .emit_instr(Instruction::Closure, Some(Rc::clone(&name)));
        self.chunk.emit_byte(up_values.len() as u8); //TODO make sure this it's convertable to u8
        for up_value in up_values {
            self.chunk.emit_byte(up_value.is_local as u8);
            self.chunk.emit_byte(up_value.idx as u8);
        }
        self.define_variable(Rc::clone(&name))?;
        Ok(())
    }

    fn var_decl(&mut self, name: Rc<Token<'a>>, initializer: &Option<Expr<'a>>) -> Result<(), ()> {
        match initializer {
            Some(expr) => self.expr(expr)?,
            None => {
                self.chunk.emit_const(Value::Nil, None)?;
            }
        };
        self.define_variable(Rc::clone(&name))
    }

    fn return_stml(&mut self, token: Rc<Token<'a>>, value: &Option<Expr<'a>>) -> Result<(), ()> {
        if !self.in_function() {
            self.error_at(token, "لا يمكنك استخدام 'أرجع' خارج دالة");
            return Err(());
        }

        match value {
            Some(expr) => {
                self.expr(&*expr)?;
            }
            None => {
                self.chunk.emit_const(Value::Nil, None)?;
            }
        }
        self.chunk.emit_instr(Instruction::Return, None);
        Ok(())
    }

    fn start_scope(&mut self) {
        self.state.borrow_mut().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let locals = self.state.borrow().locals.clone();
        let mut iter = locals.iter().rev();

        while let Some(local) = iter.next() {
            if local.depth == self.state.borrow().scope_depth {
                self.state.borrow_mut().locals.pop();
                if local.is_captured {
                    self.chunk.emit_instr(Instruction::CloseUpValue, None);
                } else {
                    self.chunk.emit_instr(Instruction::Pop, None);
                }
            } else {
                break;
            }
        }

        self.state.borrow_mut().scope_depth -= 1;
    }

    fn if_else_stml(
        &mut self,
        condition: &Expr<'a>,
        then_branch: &Box<Stml<'a>>,
        else_branch: &Option<Box<Stml<'a>>>,
    ) -> Result<(), ()> {
        self.expr(condition)?;
        let false_jump = self.chunk.emit_jump(Instruction::JumpIfFalse, None);
        self.chunk.emit_instr(Instruction::Pop, None);
        self.stml(then_branch)?;
        let true_jump = self.chunk.emit_jump(Instruction::Jump, None);
        self.chunk.patch_jump(false_jump);
        self.chunk.emit_instr(Instruction::Pop, None);
        match else_branch {
            Some(stml) => {
                self.stml(stml)?;
            }
            None => {}
        }
        self.chunk.patch_jump(true_jump);
        Ok(())
    }

    fn start_loop(&mut self) -> Loop {
        let loop_ = Loop::new(self.chunk.len());
        self.state.borrow_mut().loops.push(loop_.clone());
        loop_
    }

    fn end_loop(&mut self) {
        let loop_ = self.state.borrow_mut().loops.pop().unwrap();
        for break_ in loop_.breaks {
            self.chunk.patch_jump(break_);
        }
    }

    fn while_stml(&mut self, condition: &Expr<'a>, body: &Box<Stml<'a>>) -> Result<(), ()> {
        let start = self.start_loop().start;

        self.expr(condition)?;
        let false_jump = self.chunk.emit_jump(Instruction::JumpIfFalse, None);
        self.chunk.emit_instr(Instruction::Pop, None);
        self.stml(body)?;
        self.chunk.emit_loop(start, None);
        self.chunk.patch_jump(false_jump);
        self.chunk.emit_instr(Instruction::Pop, None);

        self.end_loop();
        Ok(())
    }

    fn loop_stml(&mut self, body: &Box<Stml<'a>>) -> Result<(), ()> {
        let start = self.start_loop().start;

        self.stml(body)?;
        self.chunk.emit_loop(start, None);

        self.end_loop();
        Ok(())
    }

    fn break_stml(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        if self.state.borrow().loops.is_empty() {
            self.error_at(token, "لا يمكنك استخدام 'قف' خارج حلقة تكرارية");
            return Err(());
        }

        let idx = self.chunk.emit_jump(Instruction::Jump, Some(token));
        self.state
            .borrow_mut()
            .loops
            .last_mut()
            .unwrap()
            .breaks
            .push(idx);
        Ok(())
    }

    fn continue_stml(&mut self, token: Rc<Token<'a>>) -> Result<(), ()> {
        if self.state.borrow().loops.is_empty() {
            self.error_at(token, "لا يمكنك استخدام 'أكمل' خارج حلقة تكرارية");
            return Err(());
        }

        let start = self.state.borrow().loops.last().unwrap().start;
        self.chunk.emit_loop(start, None);
        Ok(())
    }

    pub fn stml(&mut self, stml: &Stml<'a>) -> Result<(), ()> {
        match stml {
            Stml::Expr(expr) => {
                self.expr(expr)?;
                self.chunk.emit_instr(Instruction::Pop, None);
            }
            Stml::FunctionDecl(name, params, body) => {
                match self.function_decl(Rc::clone(name), params, body) {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.borrow_mut().had_error = true;
                        return Err(());
                    }
                };
            }
            Stml::VarDecl(name, initializer) => {
                self.var_decl(Rc::clone(name), initializer)?;
            }
            Stml::Return(token, value) => self.return_stml(Rc::clone(token), value)?,
            Stml::Throw(_, _) => unimplemented!(),
            Stml::Block(stmls) => {
                self.start_scope();
                for stml in stmls {
                    self.stml(stml)?;
                }
                self.end_scope();
            }
            Stml::IfElse(condition, then_branch, else_branch) => {
                self.if_else_stml(condition, then_branch, else_branch)?
            }
            Stml::While(condition, body) => self.while_stml(condition, body)?,
            Stml::Loop(body) => self.loop_stml(body)?,
            Stml::Break(token) => self.break_stml(Rc::clone(token))?,
            Stml::Continue(token) => self.continue_stml(Rc::clone(token))?,
            Stml::TryCatch(_, _, _) => unimplemented!(),
        }
        Ok(())
    }

    pub fn compile(&mut self) -> Result<Function<'a>, ()> {
        if cfg!(feature = "debug-bytecode") && self.typ == CompilerType::Script {
            println!("---");
            println!("[DEBUG] started compiling");
            println!("---");
        }

        for stml in self.ast {
            self.stml(stml).ok();
        }

        if self.typ == CompilerType::Function {
            self.chunk.emit_const(Value::Nil, None)?;
            self.chunk.emit_instr(Instruction::Return, None);
        }

        if self.state.borrow().had_error {
            Err(())
        } else {
            Ok(Function::new(
                self.name.clone(),
                self.chunk.clone(),
                Arity::Fixed(self.arity),
            ))
        }
    }
}
