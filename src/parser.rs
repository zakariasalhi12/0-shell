use crate::ast::*;
use crate::lexer::Token;
use crate::error::ShellError;

pub fn parse(tokens: &[Token]) -> Result<AstNode, ShellError> {
    // TODO: implement recursive descent parser
    Err(ShellError::Parse("unimplemented parser".into()))
}
