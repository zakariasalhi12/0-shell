use crate::parser::types::ArithmeticExpr;
use crate::envirement::ShellEnv;
use crate::error::ShellError;

pub fn eval_arith(_: &ArithmeticExpr, _: &mut ShellEnv) -> Result<i64, ShellError> {
    todo!();
}
