use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_pipeline(&mut self) -> Result<Option<AstNode>, ShellError> {
        let mut left = match self.parse_op()? {
            Some(command) => command,
            None => return Ok(None),
        };

        while let Some(Token::Pipe) = self.current() {
            self.advance();
            let right = match self.parse_op()? {
                Some(command) => command,
                None => {
                    return Err(ShellError::Parse(String::from(
                        "Expected command after pipe",
                    )));
                }
            };
            left = AstNode::Pipeline(vec![left, right]);
        }
        Ok(Some(left))
    }
}
