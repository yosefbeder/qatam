use super::ast::Stml;

#[cfg(feature = "debug-ast")]
pub fn debug_ast(ast: &Vec<Stml>) {
    for stml in ast {
        print!("{:?}", stml);
    }
}

#[cfg(not(feature = "debug-ast"))]
pub fn debug_ast(_: &Vec<Stml>) {}
