use super::{ast::Stml, value::Function};

#[cfg(feature = "debug-ast")]
pub fn debug_ast(ast: &Vec<Stml>) {
    for stml in ast {
        print!("{:?}", stml);
    }
}

#[cfg(not(feature = "debug-ast"))]
pub fn debug_ast(_: &Vec<Stml>) {}

#[cfg(feature = "debug-bytecode")]
pub fn debug_bytecode(function: &Function) {
    print!("{:?}", function);
}

#[cfg(not(feature = "debug-bytecode"))]
pub fn debug_bytecode(_: &Function) {}
