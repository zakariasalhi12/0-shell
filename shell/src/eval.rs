use crate::parser::types::ArithmeticExpr;
use crate::envirement::ShellEnv;
use crate::error::ShellError;

pub fn eval_arith(expr: &ArithmeticExpr, env: &mut ShellEnv) -> Result<i64, ShellError> {
    // match expr recursively and evaluate
    todo!();
}
