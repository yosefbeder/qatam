use super::{
    ast::{Expr, Literal, Stml},
    chunk::{Chunk, Instruction::*},
    parser::Parser,
    token::{Token, TokenType},
    value::{Arity, Function, Value},
};
use std::{
    cell::RefCell, ffi::OsStr, fmt, fs, io::Error as IOError, path::PathBuf, rc::Rc, result,
};

type Result = result::Result<(), ()>;

#[derive(Debug)]
enum CompileError {
    InvalidSpecialChar(Rc<Token>, Option<char>),
    TooManyConsts(Rc<Token>),
    SameVarInScope(Rc<Token>),
    TooManyParams(Rc<Token>),
    TooManyArgs(Rc<Token>),
    InvalidReturn(Rc<Token>),
    InvalidBreak(Rc<Token>),
    InvalidContinue(Rc<Token>),
    InvalidImport(Rc<Token>),
    InvalidExport(Rc<Token>),
    InvalidImportExt(Rc<Token>),
    IOError(Rc<Token>, IOError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpecialChar(token, got) => {
                if let Some(c) = got {
                    writeln!(f, "لا يمكن ل'\\' أن يكون متبوعاً ب'{c}' حيث يتبع فقط ب:")?;
                } else {
                    writeln!(f, "يجب أن يتبع '\\' بأحد هذه الخيارات:")?;
                }
                writeln!(f, "- 'n': سطر جديد")?;
                writeln!(f, "- 'r': للرجوع لبداية السطر (لا يجب عليك استخدامها إلا إذا كنت عالماً بما تفعل، وإن أردت الإستزادة عنها يمكنك البحث عن carriage return)")?;
                writeln!(f, "- 't': يوافق الضغط على مفتاح tab")?;
                writeln!(f, "- '\"': لإضافة '\"'")?;
                writeln!(f, "- '\\': لإضافة '\\'")?;
                write!(f, "{}", token.get_pos())
            }
            Self::TooManyConsts(token) => {
                writeln!(
                    f,
                    "تم إيراد الكثير من الثوابت في هذه الدالة {}",
                    token.get_pos()
                )?;
                write!(
                    f,
                    "إن حدث معك هذا الخطأ تواصل معي على dryosefbeder@gmail.com لكي أساعدك في حله"
                )
            }
            Self::SameVarInScope(token) => {
                write!(
                    f,
                    "المتغير '{}' قد تم تعريفيه في نفس المجموعة من قبل {}",
                    token.get_pos(),
                    token.lexeme
                )
            }
            Self::TooManyParams(token) | Self::TooManyArgs(token) => {
                write!(
                    f,
                    "لا يمكن أن يكون للدالة أكثر من 255 مدخل {}",
                    token.get_pos()
                )
            }
            Self::InvalidReturn(token) => {
                write!(
                    f,
                    "لا يمكن استخدام '{}' خارج الدوال {}",
                    token.get_pos(),
                    token.lexeme
                )
            }
            Self::InvalidBreak(token) | Self::InvalidContinue(token) => {
                write!(
                    f,
                    "لا يمكن استخدام '{}' خارج الحلقات التكرارية {}",
                    token.get_pos(),
                    token.lexeme
                )
            }
            Self::InvalidImport(token) => {
                write!(f, "لا يمكن الاستيراد من داخل الدوال {}", token.get_pos())
            }
            Self::InvalidExport(token) => {
                write!(f, "لا يمكن التصدير من داخل الدوال {}", token.get_pos())
            }
            Self::InvalidImportExt(token) => {
                write!(
                    f,
                    "يجب أن يكون إمتداد الملف المستورد 'قتام' {}",
                    token.get_pos()
                )
            }
            Self::IOError(token, err) => {
                writeln!(f, "حدث خطأ من نظام التشغيل {}", token.get_pos())?;
                write!(f, "{err}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CompilerType {
    Script,
    Function,
    Module,
}

#[derive(Debug, Clone)]
struct Local {
    name: Rc<Token>,
    depth: usize,
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
    fn new(name: Rc<Token>, depth: usize) -> Local {
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
            breaks: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerState {
    scope_depth: usize,
    locals: Vec<Local>, //TODO make it limited as the stack is
    up_values: Vec<UpValue>,
    had_err: bool,
    loops: Vec<Loop>,
    enclosing_state: Option<Rc<RefCell<CompilerState>>>,
}

impl CompilerState {
    fn new(enclosing_state: Option<Rc<RefCell<CompilerState>>>) -> Self {
        Self {
            scope_depth: 0,
            locals: vec![],
            up_values: vec![],
            had_err: false,
            loops: vec![],
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

    fn add_local(&mut self, token: Rc<Token>) {
        let local = Local::new(token, self.scope_depth);
        self.locals.push(local);
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

pub struct Compiler<'a> {
    typ: CompilerType,
    name: Option<String>,
    arity: u8,
    ast: &'a Vec<Stml>,
    chunk: Chunk,
    state: Rc<RefCell<CompilerState>>,
    path: Option<PathBuf>,
}

impl<'a> Compiler<'a> {
    pub fn new(ast: &'a Vec<Stml>, path: Option<PathBuf>) -> Self {
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
            path,
        }
    }

    fn new_module(ast: &'a Vec<Stml>, path: PathBuf) -> Self {
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
            path: Some(path),
        }
    }

    fn new_function(
        name: Option<String>,
        body: &'a Stml,
        enclosing_state: Rc<RefCell<CompilerState>>,
        path: Option<PathBuf>,
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
            path,
        }
    }

    fn scope_depth(&self) -> usize {
        self.state.borrow().scope_depth
    }

    fn write_const(&mut self, value: Value, token: Rc<Token>) -> Result {
        match self.chunk.write_const(value, Some(Rc::clone(&token))) {
            Ok(_) => Ok(()),
            Err(_) => Err(self.err(CompileError::TooManyConsts(token))),
        }
    }

    fn write_closure(
        &mut self,
        function: Function,
        up_values: &[UpValue],
        token: Rc<Token>,
    ) -> Result {
        match self
            .chunk
            .write_closure(function, up_values, Some(Rc::clone(&token)))
        {
            Ok(_) => Ok(()),
            Err(_) => Err(self.err(CompileError::TooManyConsts(token))),
        }
    }

    fn err(&mut self, err: CompileError) {
        eprintln!("{err}");
        self.state.borrow_mut().had_err = true;
    }

    fn quoted_string(&mut self, token: Rc<Token>) -> result::Result<String, ()> {
        let mut content = String::new();
        let mut iter = token.lexeme.chars().skip(1);
        while let Some(c) = iter.next() {
            if c == '\\' {
                if let Some(c) = iter.next() {
                    match c {
                        'n' => content.push('\n'),
                        'r' => content.push('\r'),
                        't' => content.push('\t'),
                        '\\' => content.push('\\'),
                        '"' => content.push('"'),
                        _ => {
                            self.err(CompileError::InvalidSpecialChar(token, Some(c)));
                            return Err(());
                        }
                    }
                } else {
                    self.err(CompileError::InvalidSpecialChar(token, None));
                    return Err(());
                }
            } else if c == '"' {
                break;
            } else {
                content.push(c);
            }
        }
        Ok(content)
    }

    fn string(&mut self, token: Rc<Token>) -> result::Result<String, ()> {
        if token.lexeme.starts_with('"') {
            self.quoted_string(token)
        } else {
            Ok(token.lexeme.clone())
        }
    }

    fn in_global_scope(&self) -> bool {
        self.typ == CompilerType::Script && self.scope_depth() == 0
    }

    fn in_function(&self) -> bool {
        self.typ == CompilerType::Function
    }

    /// for both imports and exports
    fn can_import(&self) -> bool {
        self.typ != CompilerType::Function && self.scope_depth() == 0
    }

    fn define_global(&mut self, token: Rc<Token>) -> Result {
        self.write_const(Value::new_string(token.lexeme.clone()), Rc::clone(&token))?;
        self.chunk.write_instr(DefineGlobal, Some(token));
        Ok(())
    }

    fn define_local(&mut self, token: Rc<Token>) -> Result {
        if &token.lexeme == "_" {
            return Ok(self.state.borrow_mut().add_local(token));
        }

        let idx = self.state.borrow().resolve_local(Rc::clone(&token));

        match idx {
            Some(idx) => {
                if self.state.borrow().locals[idx].depth == self.scope_depth() {
                    Err(self.err(CompileError::SameVarInScope(token)))
                } else {
                    Ok(self.state.borrow_mut().add_local(token))
                }
            }
            None => Ok(self.state.borrow_mut().add_local(token)),
        }
    }

    fn define_variable(&mut self, token: Rc<Token>) -> Result {
        if self.in_global_scope() {
            self.define_global(token)
        } else {
            self.define_local(token)
        }
    }

    fn set_global(&mut self, token: Rc<Token>) -> Result {
        self.write_const(Value::new_string(token.lexeme.clone()), Rc::clone(&token))?;
        self.chunk.write_instr(SetGlobal, Some(Rc::clone(&token)));
        return Ok(());
    }

    fn set_variable(&mut self, token: Rc<Token>) -> Result {
        if self.in_global_scope() {
            return self.set_global(Rc::clone(&token));
        }

        let state = self.state.borrow();
        match state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk.write_instr(SetLocal, Some(Rc::clone(&token)));
                self.chunk.write_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        let mut state = self.state.borrow_mut();
        match state.resolve_up_value(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk.write_instr(SetUpValue, Some(Rc::clone(&token)));
                self.chunk.write_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        self.set_global(Rc::clone(&token))
    }

    fn get_variable(&mut self, token: Rc<Token>) -> Result {
        let state = self.state.borrow();
        match state.resolve_local(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk.write_instr(GetLocal, Some(Rc::clone(&token)));
                self.chunk.write_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        let mut state = self.state.borrow_mut();
        match state.resolve_up_value(Rc::clone(&token)) {
            Some(idx) => {
                self.chunk.write_instr(GetUpValue, Some(Rc::clone(&token)));
                self.chunk.write_byte(idx as u8);
                return Ok(());
            }
            _ => {}
        }
        drop(state);

        self.write_const(Value::new_string(token.lexeme.clone()), Rc::clone(&token))?;
        self.chunk.write_instr(GetGlobal, Some(Rc::clone(&token)));
        Ok(())
    }

    fn literal(&mut self, literal: &Literal) -> Result {
        match literal {
            Literal::Number(token) => {
                self.write_const(
                    Value::Number(token.lexeme.clone().parse().unwrap()),
                    Rc::clone(token),
                )?;
            }
            Literal::Bool(token) => {
                self.write_const(
                    Value::Bool(match token.typ {
                        TokenType::True => true,
                        TokenType::False => false,
                        _ => unreachable!(),
                    }),
                    Rc::clone(token),
                )?;
            }
            Literal::String(token) => {
                let value = Value::new_string(self.string(Rc::clone(token))?);
                self.write_const(value, Rc::clone(token))?;
            }
            Literal::Nil(token) => {
                self.write_const(Value::Nil, Rc::clone(token))?;
            }
            Literal::List(exprs) => {
                let mut size = 0;
                for expr in exprs {
                    self.expr(expr)?;
                    size += 1;
                }
                self.chunk.write_instr(BuildList, None);
                self.chunk.write_byte(size);
            }
            Literal::Object(items) => {
                let mut size = 0;
                for item in items {
                    self.write_const(Value::new_string(item.0.lexeme.clone()), Rc::clone(&item.0))?;
                    match &item.1 {
                        Some(expr) => {
                            self.expr(expr)?;
                        }
                        None => {
                            self.get_variable(Rc::clone(&item.0))?;
                        }
                    }
                    size += 1;
                }
                self.chunk.write_instr(BuildObject, None);
                self.chunk.write_byte(size);
            }
        };
        Ok(())
    }

    fn unary(&mut self, op: Rc<Token>, expr: &Expr) -> Result {
        self.expr(expr)?;
        match op.typ {
            TokenType::Minus => {
                self.chunk.write_instr(Negate, Some(Rc::clone(&op)));
            }
            TokenType::Bang => {
                self.chunk.write_instr(Not, Some(Rc::clone(&op)));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn binary(&mut self, op: Rc<Token>, lhs: &Expr, rhs: &Expr) -> Result {
        macro_rules! set_and {
            ($instr:ident) => {{
                let token = lhs.as_variable();
                self.get_variable(token.clone())?;
                self.expr(rhs)?;
                self.chunk.write_instr($instr, Some(op));
                self.set_variable(token.clone())?;
                return Ok(());
            }};
        }

        match op.typ {
            TokenType::Equal => {
                self.expr(rhs)?;
                self.set_variable(lhs.as_variable().clone())?;
                return Ok(());
            }
            TokenType::PlusEqual => set_and!(Add),
            TokenType::MinusEqual => set_and!(Subtract),
            TokenType::StarEqual => set_and!(Multiply),
            TokenType::SlashEqual => set_and!(Divide),
            TokenType::PercentEqual => set_and!(Remainder),
            _ => {}
        }

        self.expr(lhs)?;

        match op.typ {
            TokenType::And => {
                let false_jump = self.chunk.write_jump(JumpIfFalse, Some(Rc::clone(&op)));
                self.chunk.write_instr(Pop, Some(Rc::clone(&op)));
                self.expr(rhs)?;
                self.chunk.rewrite_jump(false_jump);
                return Ok(());
            }
            TokenType::Or => {
                let true_jump = self.chunk.write_jump(JumpIfTrue, Some(Rc::clone(&op)));
                self.chunk.write_instr(Pop, Some(Rc::clone(&op)));
                self.expr(rhs)?;
                self.chunk.rewrite_jump(true_jump);
                return Ok(());
            }
            _ => {}
        }

        self.expr(rhs)?;
        match op.typ {
            TokenType::Plus => {
                self.chunk.write_instr(Add, Some(Rc::clone(&op)));
            }
            TokenType::Minus => {
                self.chunk.write_instr(Subtract, Some(Rc::clone(&op)));
            }
            TokenType::Star => {
                self.chunk.write_instr(Multiply, Some(Rc::clone(&op)));
            }
            TokenType::Slash => {
                self.chunk.write_instr(Divide, Some(Rc::clone(&op)));
            }
            TokenType::Percent => {
                self.chunk.write_instr(Remainder, Some(Rc::clone(&op)));
            }
            TokenType::DEqual => {
                self.chunk.write_instr(Equal, Some(Rc::clone(&op)));
            }
            TokenType::BangEqual => {
                self.chunk.write_instr(Equal, Some(Rc::clone(&op)));
                self.chunk.write_instr(Not, Some(Rc::clone(&op)));
            }
            TokenType::Greater => {
                self.chunk.write_instr(Greater, Some(Rc::clone(&op)));
            }
            TokenType::GreaterEqual => {
                self.chunk.write_instr(GreaterEqual, Some(Rc::clone(&op)));
            }
            TokenType::Less => {
                self.chunk.write_instr(Less, Some(Rc::clone(&op)));
            }
            TokenType::LessEqual => {
                self.chunk.write_instr(LessEqual, Some(Rc::clone(&op)));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn get(&mut self, token: Rc<Token>, instance: &Expr, key: &Expr) -> Result {
        self.expr(instance)?;
        self.expr(key)?;
        self.chunk.write_instr(Get, Some(Rc::clone(&token)));
        Ok(())
    }

    fn set(&mut self, token: Rc<Token>, instance: &Expr, key: &Expr, value: &Expr) -> Result {
        macro_rules! get_and {
            ($instr:ident) => {{
                self.expr(instance)?;
                self.expr(key)?;
                self.chunk.write_instr(Get, Some(Rc::clone(&token)));
                self.chunk.write_instr($instr, Some(Rc::clone(&token)));
            }};
        }

        self.expr(value)?;
        match token.typ {
            TokenType::PlusEqual => get_and!(Add),
            TokenType::MinusEqual => get_and!(Subtract),
            TokenType::StarEqual => get_and!(Multiply),
            TokenType::SlashEqual => get_and!(Divide),
            TokenType::PercentEqual => get_and!(Remainder),
            _ => {}
        }

        self.expr(instance)?;
        self.expr(key)?;
        self.chunk.write_instr(Set, Some(Rc::clone(&token)));
        Ok(())
    }

    fn call(&mut self, token: Rc<Token>, callee: &Expr, args: &Vec<Expr>) -> Result {
        self.expr(callee)?;
        let mut count = 0;
        for arg in args {
            if count == 0xff {
                self.err(CompileError::TooManyArgs(token));
                return Err(());
            }
            self.expr(arg)?;
            count += 1;
        }
        self.chunk.write_instr(Call, Some(Rc::clone(&token)));
        self.chunk.write_byte(count as u8);
        Ok(())
    }

    fn lambda(&mut self, token: Rc<Token>, params: &Vec<Rc<Token>>, body: &Box<Stml>) -> Result {
        let mut compiler =
            Compiler::new_function(None, body, Rc::clone(&self.state), self.path.clone());
        compiler.define_variable(Rc::new(Token::new_empty()))?;
        compiler.define_params(params)?;
        self.write_closure(
            compiler.compile().map_err(|_| {
                self.state.borrow_mut().had_err = true;
            })?,
            &compiler.state.borrow().up_values,
            token,
        )?;
        Ok(())
    }

    pub fn expr(&mut self, expr: &Expr) -> Result {
        match expr {
            Expr::Variable(token) => self.get_variable(Rc::clone(token))?,
            Expr::Literal(literal) => self.literal(literal)?,
            Expr::Unary(op, expr) => self.unary(Rc::clone(op), expr)?,
            Expr::Binary(op, lhs, rhs) => self.binary(Rc::clone(op), lhs, rhs)?,
            Expr::Get(token, instance, key) => self.get(Rc::clone(&token), instance, key)?,
            Expr::Set(token, instance, key, value) => {
                self.set(Rc::clone(&token), instance, key, value)?
            }
            Expr::Call(token, callee, args) => self.call(Rc::clone(&token), callee, args)?,
            Expr::Lambda(token, params, body) => self.lambda(Rc::clone(token), params, body)?,
        };
        Ok(())
    }

    fn define_params(&mut self, params: &Vec<Rc<Token>>) -> Result {
        if self.typ == CompilerType::Script {
            unreachable!();
        }

        for param in params {
            if self.arity == 0xff {
                self.err(CompileError::TooManyParams(Rc::clone(param)));
                return Err(());
            }
            self.define_variable(Rc::clone(param))?;
            self.arity += 1;
        }

        Ok(())
    }

    fn function_decl(&mut self, name: Rc<Token>, params: &Vec<Rc<Token>>, body: &Stml) -> Result {
        let mut compiler = Compiler::new_function(
            Some(name.lexeme.clone()),
            body,
            Rc::clone(&self.state),
            self.path.clone(),
        );
        compiler.define_variable(Rc::clone(&name))?;
        compiler.define_params(params)?;
        self.write_closure(
            compiler.compile().map_err(|_| {
                self.state.borrow_mut().had_err = true;
            })?,
            &compiler.state.borrow().up_values,
            Rc::clone(&name),
        )?;
        self.define_variable(name)?;
        Ok(())
    }
    fn var_decl(
        &mut self,
        token: Rc<Token>,
        name: Rc<Token>,
        initializer: &Option<Expr>,
    ) -> Result {
        match initializer {
            Some(expr) => self.expr(expr)?,
            None => {
                self.write_const(Value::Nil, token)?;
            }
        };
        self.define_variable(Rc::clone(&name))
    }

    fn return_stml(&mut self, token: Rc<Token>, value: &Option<Expr>) -> Result {
        if !self.in_function() {
            self.err(CompileError::InvalidReturn(token));
            return Err(());
        }
        match value {
            Some(expr) => {
                self.expr(&*expr)?;
            }
            None => {
                self.write_const(Value::Nil, token)?;
            }
        }
        self.chunk.write_instr(Return, None);
        Ok(())
    }

    fn throw_stml(&mut self, token: Rc<Token>, value: &Option<Expr>) -> Result {
        match value {
            Some(expr) => {
                self.expr(&*expr)?;
            }
            None => {
                self.write_const(Value::Nil, Rc::clone(&token))?;
            }
        }
        self.chunk.write_instr(Throw, Some(token));
        Ok(())
    }

    fn start_scope(&mut self) {
        self.state.borrow_mut().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let locals = self.state.borrow().locals.clone();
        let mut iter = locals.iter().rev();

        while let Some(local) = iter.next() {
            if local.depth == self.scope_depth() {
                self.state.borrow_mut().locals.pop();
                if local.is_captured {
                    self.chunk.write_instr(CloseUpValue, None);
                } else {
                    self.chunk.write_instr(Pop, None);
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
        if_body: &Box<Stml>,
        elseifs: &Vec<(Expr, Stml)>,
        else_body: &Option<Box<Stml>>,
    ) -> Result {
        self.expr(condition)?;
        let false_jump = self.chunk.write_jump(JumpIfFalse, None);
        self.chunk.write_instr(Pop, None);
        self.stml(if_body)?;
        let mut true_jumps = vec![self.chunk.write_jump(Jump, None)];
        self.chunk.rewrite_jump(false_jump);
        self.chunk.write_instr(Pop, None);
        let mut iter = elseifs.iter();
        while let Some((condition, body)) = iter.next() {
            self.expr(condition)?;
            let false_jump = self.chunk.write_jump(JumpIfFalse, None);
            self.chunk.write_instr(Pop, None);
            self.stml(body)?;
            true_jumps.push(self.chunk.write_jump(Jump, None));
            self.chunk.rewrite_jump(false_jump);
            self.chunk.write_instr(Pop, None);
        }
        match else_body {
            Some(stml) => {
                self.stml(stml)?;
            }
            None => {}
        }
        for jump in true_jumps {
            self.chunk.rewrite_jump(jump)
        }
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
            self.chunk.rewrite_jump(break_);
        }
    }

    fn while_stml(&mut self, condition: &Expr, body: &Box<Stml>) -> Result {
        let start = self.start_loop().start;

        self.expr(condition)?;
        let false_jump = self.chunk.write_jump(JumpIfFalse, None);
        self.chunk.write_instr(Pop, None);
        self.stml(body)?;
        self.chunk.write_loop(start, None);
        self.chunk.rewrite_jump(false_jump);
        self.chunk.write_instr(Pop, None);

        self.end_loop();
        Ok(())
    }

    fn loop_stml(&mut self, body: &Box<Stml>) -> Result {
        let start = self.start_loop().start;

        self.stml(body)?;
        self.chunk.write_loop(start, None);

        self.end_loop();
        Ok(())
    }

    fn break_stml(&mut self, token: Rc<Token>) -> Result {
        if self.state.borrow().loops.is_empty() {
            self.err(CompileError::InvalidBreak(token));
            return Err(());
        }

        let idx = self.chunk.write_jump(Jump, Some(token));
        self.state.borrow_mut().last_loop_mut().breaks.push(idx);
        Ok(())
    }

    fn continue_stml(&mut self, token: Rc<Token>) -> Result {
        if self.state.borrow().loops.is_empty() {
            self.err(CompileError::InvalidContinue(token));
            return Err(());
        }

        let start = self.state.borrow().last_loop().start;
        self.chunk.write_loop(start, None);
        Ok(())
    }

    /// creates a path out of the path of the current compiler and the string stored in token
    fn path(&mut self, token: Rc<Token>) -> result::Result<PathBuf, ()> {
        let temp = self.string(token)?;
        if let Some(cur_path) = &self.path {
            if let Some(dir) = cur_path.parent() {
                return Ok(dir.join(temp));
            }
        }
        Ok(PathBuf::from(temp))
    }

    fn import_stml(&mut self, name: Rc<Token>, path_token: Rc<Token>) -> Result {
        if !self.can_import() {
            self.err(CompileError::InvalidImport(name));
            return Err(());
        }

        let path = self.path(Rc::clone(&path_token))?;

        if path.extension() != Some(OsStr::new("قتام")) {
            self.err(CompileError::InvalidImportExt(Rc::clone(&path_token)));
            return Err(());
        }

        let source = fs::read_to_string(&path)
            .map_err(|err| self.err(CompileError::IOError(path_token, err)))?;
        let mut parser = Parser::new(source, Some(path.clone()));
        let ast = parser.parse().map_err(|_| {
            self.state.borrow_mut().had_err = true;
        })?;
        let mut compiler = Compiler::new_module(&ast, path);
        self.write_closure(
            compiler.compile().map_err(|_| {
                self.state.borrow_mut().had_err = true;
            })?,
            &compiler.state.borrow().up_values,
            Rc::clone(&name),
        )?;
        self.chunk.write_instr(Call, Some(Rc::clone(&name)));
        self.chunk.write_byte(0u8);
        self.define_variable(name)
    }

    pub fn export_stml(&mut self, token: Rc<Token>, stml: &Stml) -> Result {
        if !self.can_import() {
            self.err(CompileError::InvalidExport(token));
            return Err(());
        }

        match stml {
            Stml::FunctionDecl(name, params, body) => {
                match self.function_decl(Rc::clone(name), params, body) {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.borrow_mut().had_err = true;
                        return Err(());
                    }
                }
            }
            Stml::VarDecl(token, name, initializer) => {
                self.var_decl(Rc::clone(token), Rc::clone(name), initializer)?;
            }
            _ => unreachable!(),
        };
        self.state.borrow_mut().last_local_mut().export();
        Ok(())
    }

    fn try_catch_stml(
        &mut self,
        try_block: &Box<Stml>,
        err: Rc<Token>,
        catch_block: &Box<Stml>,
    ) -> Result {
        let catch_jump = self.chunk.write_jump(AppendHandler, None);
        self.stml(try_block)?;
        self.chunk.write_instr(PopHandler, None);
        let finally_jump = self.chunk.write_jump(Jump, None);
        self.chunk.rewrite_jump(catch_jump);

        match &**catch_block {
            Stml::Block(stmls) => {
                self.start_scope();
                self.define_variable(err)?;
                for stml in stmls {
                    self.stml(stml)?;
                }
                self.end_scope();
            }
            _ => unreachable!(),
        }

        self.chunk.rewrite_jump(finally_jump);
        Ok(())
    }

    fn for_in_stml(
        &mut self,
        token: Rc<Token>,
        element: Rc<Token>,
        iterator: &Expr,
        body: &Box<Stml>,
    ) -> Result {
        // 1. Append the counter and store it's stack idx
        self.start_scope();
        let counter_idx = self.state.borrow().locals.len();
        macro_rules! get_counter {
            () => {{
                self.chunk.write_instr(GetLocal, None);
                self.chunk.write_byte(counter_idx as u8);
            }};
        }
        macro_rules! increase_counter {
            () => {{
                get_counter!();
                self.write_const(Value::Number(1.0), Rc::clone(&token))?;
                self.chunk.write_instr(Add, None);
                self.chunk.write_instr(SetLocal, None);
                self.chunk.write_byte(counter_idx as u8);
                self.chunk.write_instr(Pop, None);
            }};
        }
        self.write_const(Value::Number(0.0), Rc::clone(&token))?;
        self.define_variable(Rc::new(Token::new_empty()))?;

        // 2. Check the next element
        let start = self.chunk.len();
        self.expr(iterator)?;
        self.chunk.write_instr(Size, Some(Rc::clone(&element))); //TODO find a better token to report this err
        get_counter!();
        self.chunk.write_instr(Greater, None);

        let false_jump = self.chunk.write_jump(JumpIfFalse, None);
        self.chunk.write_instr(Pop, None);

        // 3. Compile the block
        self.start_scope();
        self.expr(iterator)?;
        get_counter!();
        self.chunk.write_instr(Get, None);
        self.define_variable(Rc::clone(&element))?;

        for stml in body.as_block() {
            self.stml(stml)?;
        }

        increase_counter!();
        self.end_scope();
        self.chunk.write_loop(start, None);
        self.chunk.rewrite_jump(false_jump);
        self.chunk.write_instr(Pop, None);
        self.end_scope();
        Ok(())
    }

    pub fn stml(&mut self, stml: &Stml) -> Result {
        match stml {
            Stml::Expr(expr) => {
                self.expr(expr)?;
                self.chunk.write_instr(Pop, None);
            }
            Stml::FunctionDecl(name, params, body) => {
                self.function_decl(Rc::clone(name), params, body)?
            }
            Stml::VarDecl(token, name, initializer) => {
                self.var_decl(Rc::clone(token), Rc::clone(name), initializer)?;
            }
            Stml::Return(token, value) => self.return_stml(Rc::clone(token), value)?,
            Stml::Throw(token, value) => self.throw_stml(Rc::clone(token), value)?,
            Stml::Block(stmls) => {
                self.start_scope();
                for stml in stmls {
                    self.stml(stml)?;
                }
                self.end_scope();
            }
            Stml::IfElse(condition, if_body, elseifs, else_body) => {
                self.if_else_stml(condition, if_body, elseifs, else_body)?
            }
            Stml::While(condition, body) => self.while_stml(condition, body)?,
            Stml::Loop(body) => self.loop_stml(body)?,
            Stml::Break(token) => self.break_stml(Rc::clone(token))?,
            Stml::Continue(token) => self.continue_stml(Rc::clone(token))?,
            Stml::Import(name, path) => self.import_stml(Rc::clone(name), Rc::clone(path))?,
            Stml::Export(token, stml) => self.export_stml(Rc::clone(token), stml)?,
            Stml::TryCatch(try_block, err, catch_block) => {
                self.try_catch_stml(try_block, Rc::clone(err), catch_block)?
            }
            Stml::ForIn(token, element, iterator, body) => {
                self.for_in_stml(Rc::clone(token), Rc::clone(element), iterator, body)?
            }
        }
        Ok(())
    }

    pub fn compile(&mut self) -> result::Result<Function, ()> {
        if cfg!(feature = "debug-bytecode") && self.typ == CompilerType::Script {
            println!("---");
            println!("[DEBUG] started compiling");
            println!("---");
        }

        for stml in self.ast {
            self.stml(stml).ok();
        }

        match self.typ {
            CompilerType::Function => {
                self.write_const(Value::Nil, Rc::new(Token::new_empty()))?; //? not gonna be used anyways 😅
                self.chunk.write_instr(Return, None);
            }
            CompilerType::Module => {
                let state = &self.state.borrow();
                let mut sum = 0;

                for idx in 1..state.locals.len() {
                    let local = state.get_local(idx);
                    if local.is_exported {
                        self.chunk
                            .write_const(Value::new_string(local.name.lexeme.clone()), None)?;
                        self.chunk.write_instr(GetLocal, None);
                        self.chunk.write_byte(idx as u8);
                        sum += 1;
                    }
                }

                self.chunk.write_instr(BuildObject, None);
                self.chunk.write_byte(sum as u8);
                self.chunk.write_instr(Return, None);
            }
            _ => {}
        }

        if self.state.borrow().had_err {
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
