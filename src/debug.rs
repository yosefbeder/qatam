use super::{ast::Stml, chunk::Chunk, value::Function, vm::Frame};

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

#[cfg(feature = "debug-execution")]
pub fn debug_ip(chunk: &Chunk, offset: usize) {
    print!("{}", chunk.disassemble_instr_at(offset, false).0);
}

#[cfg(not(feature = "debug-execution"))]
pub fn debug_ip(_: &Chunk, _: usize) {}

#[cfg(feature = "debug-execution")]
pub fn debug_call(frame: &Frame) {
    println!("[DEBUG] called {:?}", frame)
}

#[cfg(not(feature = "debug-execution"))]
pub fn debug_call(_: &Frame) {}

#[cfg(feature = "debug-execution")]
pub fn debug_return(frame: Frame, cur_frame: &Frame) {
    let mut buffer = String::new();

    buffer += format!("[DEBUG] returned from {:?}\n", frame).as_str();
    buffer += format!("|       to {:?}", cur_frame).as_str();

    println!("{}", buffer);
}

#[cfg(not(feature = "debug-execution"))]
pub fn debug_return(_: Frame, _: &Frame) {}
