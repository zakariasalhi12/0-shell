use crate::parser::types::*;
use crate::env::ShellEnv;
use crate::error::ShellError;

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    // pattern match on AstNode and dispatch behavior
    todo!()
}
