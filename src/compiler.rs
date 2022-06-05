use super::{
    ast::{Expr, Literal, Stml},
    chunk::{Chunk, Instruction},
    parser::Parser,
    path::{get_dir, get_path},
    reporter::{Phase, Report, Reporter},
    token::{Token, TokenType},
    tokenizer::Tokenizer,
    value::{Arity, Function, Value},
};
use std::{
    cell::RefCell,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CompilerType {
    Script,
    Function,
    Module,
}

#[derive(Debug, Clone)]
struct Local {
    name: Rc<Token>,
    depth: u32,
    is_captured: bool,
    is_exported: bool,
}

#[derive(Debug, Clone)]
pub struct UpValue {
    pub is_local: bool,
    pub idx: usize,
}

impl UpValue {
    fn new(is_local: bool, idx: usize) -> Self {
        Self { is_local, idx }
    }
}

impl Local {
    fn new(name: Rc<Token>, depth: u32) -> Local {
        Local {
            name,
            depth,
            is_captured: false,
            is_exported: false,
        }
    }

    fn capture(&mut self) {
        self.is_captured = true;
    }

    fn export(&mut self) {
        self.is_exported = true;
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
pub struct CompilerState {
    scope_depth: u32,
    locals: Vec<Local>, //TODO make it limited as the stack is
    up_values: Vec<UpValue>,
    had_error: bool,
    loops: Vec<Loop>,
    enclosing_state: Option<Rc<RefCell<CompilerState>>>,
}

impl CompilerState {
    fn new(enclosing_state: Option<Rc<RefCell<CompilerState>>>) -> Self {
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

    fn resolve_local(&self, token: Rc<Token>) -> Option<usize> {
        let mut iter = self.locals.iter().enumerate().rev();

        while let Some((idx, local)) = iter.next() {
            if local.name == token {
                return Some(idx);
            }
        }

        None
    }

    fn resolve_up_value(&mut self, token: Rc<Token>) -> Option<usize> {
        if self.enclosing_state.is_none() {
            return None;
        }

        let mut enclosing_state = self.enclosing_state.as_ref().unwrap().borrow_mut();
        match enclosing_state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                enclosing_state.get_local_mut(idx).capture();
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

    fn get_local(&self, idx: usize) -> &Local {
        self.locals.get(idx).unwrap()
    }

    fn get_local_mut(&mut self, idx: usize) -> &mut Local {
        self.locals.get_mut(idx).unwrap()
    }

    fn last_local_mut(&mut self) -> &mut Local {
        self.locals.last_mut().unwrap()
    }

    fn last_loop(&self) -> &Loop {
        self.loops.last().unwrap()
    }

    fn last_loop_mut(&mut self) -> &mut Loop {
        self.loops.last_mut().unwrap()
    }
}

pub struct Compiler<'b> {
    typ: CompilerType,
    name: Option<String>,
    arity: u8,
    ast: &'b Vec<Stml>,
    chunk: Chunk,
    state: Rc<RefCell<CompilerState>>,
    cwd: PathBuf,
}

impl<'b> Compiler<'b> {
    pub fn new(ast: &'b Vec<Stml>) -> Self {
        let mut state = CompilerState::new(None);

        state
            .locals
            .push(Local::new(Rc::new(Token::new_empty()), 0));

        Compiler {
            typ: CompilerType::Script,
            name: None,
            arity: 0,
            ast,
            chunk: Chunk::new(),
            state: Rc::new(RefCell::new(state)),
            cwd: env::current_dir().unwrap(),
        }
    }

    fn new_module(ast: &'b Vec<Stml>, cwd: &Path) -> Self {
        let mut state = CompilerState::new(None);

        state
            .locals
            .push(Local::new(Rc::new(Token::new_empty()), 0));

        Compiler {
            typ: CompilerType::Module,
            name: None,
            arity: 0,
            ast,
            chunk: Chunk::new(),
            state: Rc::new(RefCell::new(state)),
            cwd: cwd.to_owned(),
        }
    }

    fn new_function(
        name: Option<String>,
        body: &'b Stml,
        enclosing_state: Rc<RefCell<CompilerState>>,
        cwd: &Path,
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
            state: Rc::new(RefCell::new(CompilerState::new(Some(enclosing_state)))),
            cwd: cwd.to_owned(),
        }
    }

    fn error_at(&mut self, token: Rc<Token>, msg: &str, reporter: &mut dyn Reporter) {
        let report = Report::new(Phase::Compilation, msg.to_string(), token);
        reporter.error(report);
        self.state.borrow_mut().had_error = true;
    }

    // fn warning_at(&self, token: &Token<>, msg: &str) {
    //     let report = Report::new(Phase::Parsing, msg.to_string(), Rc::new(token.clone()));
    //     reporter.warning(report);
    // }

    fn string(&mut self, token: Rc<Token>, reporter: &mut dyn Reporter) -> Result<String, ()> {
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
                            self.error_at(Rc::clone(&token), "رمز غير متوقع بعد '\\'", reporter);
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
        Ok(content)
    }

    fn in_global_scope(&self) -> bool {
        self.typ == CompilerType::Script && self.state.borrow().scope_depth == 0
    }

    fn in_function(&self) -> bool {
        self.typ == CompilerType::Function
    }

    /// for both imports and exports
    fn can_import(&self) -> bool {
        self.typ != CompilerType::Function && self.state.borrow().scope_depth == 0
    }

    fn define_variable(&mut self, token: Rc<Token>, reporter: &mut dyn Reporter) -> Result<(), ()> {
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
                    reporter,
                );
                return Err(());
            }
        }

        let local = Local::new(token, scope_depth);
        self.state.borrow_mut().locals.push(local);
        Ok(())
    }

    fn set_global(&mut self, token: Rc<Token>) -> Result<(), ()> {
        self.chunk
            .emit_const(Value::String(token.lexeme.clone()), Some(Rc::clone(&token)))?;
        self.chunk
            .emit_instr(Instruction::SetGlobal, Some(Rc::clone(&token)));
        return Ok(());
    }

    fn set_variable(&mut self, token: Rc<Token>) -> Result<(), ()> {
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

    fn get_variable(&mut self, token: Rc<Token>) -> Result<(), ()> {
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

    fn literal(&mut self, literal: &Literal, reporter: &mut dyn Reporter) -> Result<(), ()> {
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
                let value = Value::String(self.string(Rc::clone(token), reporter)?);
                self.chunk.emit_const(value, Some(Rc::clone(token)))?;
            }
            Literal::Nil(token) => {
                self.chunk.emit_const(Value::Nil, Some(Rc::clone(token)))?;
            }
            Literal::List(exprs) => {
                let mut size = 0;
                for expr in exprs {
                    self.expr(expr, reporter)?;
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
                    self.expr(&item.1, reporter)?;
                    size += 1;
                }
                self.chunk.emit_instr(Instruction::BuildObject, None);
                self.chunk.emit_byte(size);
            }
        };
        Ok(())
    }

    fn unary(&mut self, op: Rc<Token>, expr: &Expr, reporter: &mut dyn Reporter) -> Result<(), ()> {
        self.expr(expr, reporter)?;
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

    fn binary(
        &mut self,
        op: Rc<Token>,
        left: &Expr,
        right: &Expr,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if op.typ == TokenType::Equal {
            match left {
                Expr::Variable(token) => {
                    self.expr(right, reporter)?;
                    self.set_variable(Rc::clone(token))?;
                }
                _ => unreachable!(),
            }
            return Ok(());
        }

        self.expr(left, reporter)?;

        match op.typ {
            TokenType::And => {
                let false_jump = self
                    .chunk
                    .emit_jump(Instruction::JumpIfFalse, Some(Rc::clone(&op)));
                self.chunk
                    .emit_instr(Instruction::Pop, Some(Rc::clone(&op)));
                self.expr(right, reporter)?;
                self.chunk.patch_jump(false_jump);
                return Ok(());
            }
            TokenType::Or => {
                let true_jump = self
                    .chunk
                    .emit_jump(Instruction::JumpIfTrue, Some(Rc::clone(&op)));
                self.chunk
                    .emit_instr(Instruction::Pop, Some(Rc::clone(&op)));
                self.expr(right, reporter)?;
                self.chunk.patch_jump(true_jump);
                return Ok(());
            }
            _ => {}
        }

        self.expr(right, reporter)?;
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
    fn get(
        &mut self,
        token: Rc<Token>,
        instance: &Expr,
        key: &Expr,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        self.expr(instance, reporter)?;
        self.expr(key, reporter)?;
        self.chunk
            .emit_instr(Instruction::Get, Some(Rc::clone(&token)));
        Ok(())
    }

    fn set(
        &mut self,
        token: Rc<Token>,
        instance: &Expr,
        key: &Expr,
        value: &Expr,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        self.expr(value, reporter)?;
        self.expr(instance, reporter)?;
        self.expr(key, reporter)?;
        self.chunk
            .emit_instr(Instruction::Set, Some(Rc::clone(&token)));
        Ok(())
    }

    fn call(
        &mut self,
        token: Rc<Token>,
        callee: &Expr,
        args: &Vec<Expr>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        self.expr(callee, reporter)?;
        let mut count = 0;
        for arg in args {
            if count == 0xff {
                self.error_at(token, "عدد كثر من المدخلات", reporter);
                return Err(());
            }
            self.expr(arg, reporter)?;
            count += 1;
        }
        self.chunk
            .emit_instr(Instruction::Call, Some(Rc::clone(&token)));
        self.chunk.emit_byte(count as u8);
        Ok(())
    }

    pub fn expr(&mut self, expr: &Expr, reporter: &mut dyn Reporter) -> Result<(), ()> {
        match expr {
            Expr::Variable(token) => self.get_variable(Rc::clone(token))?,
            Expr::Literal(literal) => self.literal(literal, reporter)?,
            Expr::Unary(op, expr) => self.unary(Rc::clone(op), expr, reporter)?,
            Expr::Binary(op, left, right) => self.binary(Rc::clone(op), left, right, reporter)?,
            Expr::Get(token, instance, key) => {
                self.get(Rc::clone(&token), instance, key, reporter)?
            }
            Expr::Set(token, instance, key, value) => {
                self.set(Rc::clone(&token), instance, key, value, reporter)?
            }
            Expr::Call(token, callee, args) => {
                self.call(Rc::clone(&token), callee, args, reporter)?
            }
        };
        Ok(())
    }

    fn define_params(
        &mut self,
        params: &Vec<Rc<Token>>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if self.typ == CompilerType::Script {
            unreachable!();
        }

        for param in params {
            if self.arity == 0xff {
                self.error_at(Rc::clone(param), "عدد كثير من المعاملات", reporter);
                return Err(());
            }
            self.define_variable(Rc::clone(param), reporter)?;
            self.arity += 1;
        }

        Ok(())
    }

    fn function_decl(
        &mut self,
        name: Rc<Token>,
        params: &Vec<Rc<Token>>,
        body: &Stml,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        let mut compiler = Compiler::new_function(
            Some(name.lexeme.clone()),
            body,
            Rc::clone(&self.state),
            &self.cwd,
        );
        compiler.define_variable(Rc::clone(&name), reporter)?;
        compiler.define_params(params, reporter)?;
        self.chunk.emit_closure(
            compiler.compile(reporter)?,
            &compiler.state.borrow().up_values,
            Rc::clone(&name),
        )?;
        self.define_variable(Rc::clone(&name), reporter)?;
        Ok(())
    }
    fn var_decl(
        &mut self,
        name: Rc<Token>,
        initializer: &Option<Expr>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        match initializer {
            Some(expr) => self.expr(expr, reporter)?,
            None => {
                self.chunk.emit_const(Value::Nil, None)?;
            }
        };
        self.define_variable(Rc::clone(&name), reporter)
    }

    fn return_stml(
        &mut self,
        token: Rc<Token>,
        value: &Option<Expr>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if !self.in_function() {
            self.error_at(token, "لا يمكنك استخدام 'أرجع' خارج دالة", reporter);
            return Err(());
        }

        match value {
            Some(expr) => {
                self.expr(&*expr, reporter)?;
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
        condition: &Expr,
        then_branch: &Box<Stml>,
        else_branch: &Option<Box<Stml>>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        self.expr(condition, reporter)?;
        let false_jump = self.chunk.emit_jump(Instruction::JumpIfFalse, None);
        self.chunk.emit_instr(Instruction::Pop, None);
        self.stml(then_branch, reporter)?;
        let true_jump = self.chunk.emit_jump(Instruction::Jump, None);
        self.chunk.patch_jump(false_jump);
        self.chunk.emit_instr(Instruction::Pop, None);
        match else_branch {
            Some(stml) => {
                self.stml(stml, reporter)?;
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

    fn while_stml(
        &mut self,
        condition: &Expr,
        body: &Box<Stml>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        let start = self.start_loop().start;

        self.expr(condition, reporter)?;
        let false_jump = self.chunk.emit_jump(Instruction::JumpIfFalse, None);
        self.chunk.emit_instr(Instruction::Pop, None);
        self.stml(body, reporter)?;
        self.chunk.emit_loop(start, None);
        self.chunk.patch_jump(false_jump);
        self.chunk.emit_instr(Instruction::Pop, None);

        self.end_loop();
        Ok(())
    }

    fn loop_stml(&mut self, body: &Box<Stml>, reporter: &mut dyn Reporter) -> Result<(), ()> {
        let start = self.start_loop().start;

        self.stml(body, reporter)?;
        self.chunk.emit_loop(start, None);

        self.end_loop();
        Ok(())
    }

    fn break_stml(&mut self, token: Rc<Token>, reporter: &mut dyn Reporter) -> Result<(), ()> {
        if self.state.borrow().loops.is_empty() {
            self.error_at(token, "لا يمكنك استخدام 'قف' خارج حلقة تكرارية", reporter);
            return Err(());
        }

        let idx = self.chunk.emit_jump(Instruction::Jump, Some(token));
        self.state.borrow_mut().last_loop_mut().breaks.push(idx);
        Ok(())
    }

    fn continue_stml(&mut self, token: Rc<Token>, reporter: &mut dyn Reporter) -> Result<(), ()> {
        if self.state.borrow().loops.is_empty() {
            self.error_at(token, "لا يمكنك استخدام 'أكمل' خارج حلقة تكرارية", reporter);
            return Err(());
        }

        let start = self.state.borrow().last_loop().start;
        self.chunk.emit_loop(start, None);
        Ok(())
    }

    fn import_stml(
        &mut self,
        name: Rc<Token>,
        path: Rc<Token>,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if !self.can_import() {
            self.error_at(name, "لا يمكنك الاستيراد في هذا السياق", reporter);
            return Err(());
        }

        let path = self.string(path, reporter)?;
        let path = get_path(&self.cwd, &path).or_else(|err| {
            self.error_at(Rc::clone(&name), &err, reporter);
            return Err(());
        })?;
        let source = fs::read_to_string(&path).unwrap();
        let mut tokenizer = Tokenizer::new(source, Some(&path));
        let mut parser = Parser::new(&mut tokenizer);
        let ast = parser.parse(reporter)?;
        let mut compiler = Compiler::new_module(&ast, &get_dir(&path));
        self.chunk.emit_closure(
            compiler.compile(reporter)?,
            &compiler.state.borrow().up_values,
            Rc::clone(&name),
        )?;
        self.chunk
            .emit_instr(Instruction::Call, Some(Rc::clone(&name)));
        self.chunk.emit_byte(0u8);
        self.define_variable(name, reporter)
    }

    pub fn export_stml(
        &mut self,
        token: Rc<Token>,
        stml: &Stml,
        reporter: &mut dyn Reporter,
    ) -> Result<(), ()> {
        if !self.can_import() {
            self.error_at(token, "لا يمكنك التصدير في هذا السياق", reporter);
            return Err(());
        }

        match stml {
            Stml::FunctionDecl(name, params, body) => {
                match self.function_decl(Rc::clone(name), params, body, reporter) {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.borrow_mut().had_error = true;
                        return Err(());
                    }
                }
            }
            Stml::VarDecl(name, initializer) => {
                self.var_decl(Rc::clone(name), initializer, reporter)?;
            }
            _ => unreachable!(),
        };
        self.state.borrow_mut().last_local_mut().export();
        Ok(())
    }

    pub fn stml(&mut self, stml: &Stml, reporter: &mut dyn Reporter) -> Result<(), ()> {
        match stml {
            Stml::Expr(expr) => {
                self.expr(expr, reporter)?;
                self.chunk.emit_instr(Instruction::Pop, None);
            }
            Stml::FunctionDecl(name, params, body) => {
                match self.function_decl(Rc::clone(name), params, body, reporter) {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.borrow_mut().had_error = true;
                        return Err(());
                    }
                };
            }
            Stml::VarDecl(name, initializer) => {
                self.var_decl(Rc::clone(name), initializer, reporter)?;
            }
            Stml::Return(token, value) => self.return_stml(Rc::clone(token), value, reporter)?,
            Stml::Throw(_, _) => unimplemented!(),
            Stml::Block(stmls) => {
                self.start_scope();
                for stml in stmls {
                    self.stml(stml, reporter)?;
                }
                self.end_scope();
            }
            Stml::IfElse(condition, then_branch, else_branch) => {
                self.if_else_stml(condition, then_branch, else_branch, reporter)?
            }
            Stml::While(condition, body) => self.while_stml(condition, body, reporter)?,
            Stml::Loop(body) => self.loop_stml(body, reporter)?,
            Stml::Break(token) => self.break_stml(Rc::clone(token), reporter)?,
            Stml::Continue(token) => self.continue_stml(Rc::clone(token), reporter)?,
            Stml::Import(name, path) => {
                self.import_stml(Rc::clone(name), Rc::clone(path), reporter)?
            }
            Stml::Export(token, stml) => self.export_stml(Rc::clone(token), stml, reporter)?,
            Stml::TryCatch(_, _, _) => unimplemented!(),
        }
        Ok(())
    }

    pub fn compile(&mut self, reporter: &mut dyn Reporter) -> Result<Function, ()> {
        if cfg!(feature = "debug-bytecode") && self.typ == CompilerType::Script {
            println!("---");
            println!("[DEBUG] started compiling");
            println!("---");
        }

        for stml in self.ast {
            self.stml(stml, reporter).ok();
        }

        match self.typ {
            CompilerType::Function => {
                self.chunk.emit_const(Value::Nil, None)?;
                self.chunk.emit_instr(Instruction::Return, None);
            }
            CompilerType::Module => {
                let state = &self.state.borrow();
                let mut sum = 0;

                for idx in 1..state.locals.len() {
                    let local = state.get_local(idx);
                    if local.is_exported {
                        self.chunk
                            .emit_const(Value::String(local.name.lexeme.clone()), None)?;
                        self.chunk.emit_instr(Instruction::GetLocal, None);
                        self.chunk.emit_byte(idx as u8);
                        sum += 1;
                    }
                }

                self.chunk.emit_instr(Instruction::BuildObject, None);
                self.chunk.emit_byte(sum as u8);
                self.chunk.emit_instr(Instruction::Return, None);
            }
            _ => {}
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
