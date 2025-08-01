use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_op(&mut self) -> Result<Option<AstNode>, ShellError> {

        let should_negate = match self.current() {
            Some(Token::LogicalNot) =>{
                self.advance();
                true
            },
            _ => false
        };
        
        let mut left = match self.parse_command()? {
            Some(command) =>{
                if should_negate{
                    AstNode::Not(Box::new(command))
                }else{
                    command
                }
            },
            None => return Ok(None),
        };


        while let Some(token) = self.current() {
            match token {
                Token::LogicalAnd => {
                    self.advance();
                    let right = match self.parse_command()? {
                        Some(command) => command,
                        None => {
                            return Err(ShellError::Parse(String::from(
                                "Expected command after &&",
                            )));
                        }
                    };
                    left = AstNode::And(Box::new(left), Box::new(right));
                }

                Token::LogicalOr => {
                    self.advance();
                    let right = match self.parse_op()? {
                        Some(command) => command,
                        None => {
                            return Err(ShellError::Parse(String::from(
                                "Expected command after ||",
                            )));
                        }
                    };
                    left = AstNode::Or(Box::new(left), Box::new(right));
                }

                _ => break,
            }
        }
        Ok(Some(left))
    }
}
