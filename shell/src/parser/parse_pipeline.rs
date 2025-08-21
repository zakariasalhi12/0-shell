use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_pipeline(&mut self) -> Result<Option<AstNode>, ShellError> {
        let mut commands = Vec::new();

        let first_command = match self.parse_op()? {
            Some(command) => command,
            None => return Ok(None),
        };
        commands.push(first_command);

        while let Some(Token::Pipe) = self.current() {
            self.advance();
            let next_command = match self.parse_op()? {
                Some(command) => command,
                None => {
                    return Err(ShellError::Parse(String::from(
                        "Expected command after pipe",
                    )));
                }
            };
            commands.push(next_command);
        }

        if commands.len() == 1{
            return Ok(Some(commands[0].clone()));
        }
        Ok(Some(AstNode::Pipeline(commands)))
    }
}
