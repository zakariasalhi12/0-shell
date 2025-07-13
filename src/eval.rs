use crate::ast::ArithmeticExpr;
use crate::env::ShellEnv;
use crate::error::ShellError;

pub fn eval_arith(expr: &ArithmeticExpr, env: &mut ShellEnv) -> Result<i64, ShellError> {
    // match expr recursively and evaluate
    todo!();
}
