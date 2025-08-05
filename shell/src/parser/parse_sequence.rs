use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_sequence(&mut self) -> Result<Option<AstNode>, ShellError> {
        let mut commands = Vec::new();

        loop {
            if let Some(cmd) = self.parse_pipeline()? {
                commands.push(cmd);
            } else {
                break;
            }

            match self.current() {
                Some(Token::Semicolon) => {
                    self.advance();
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                Some(Token::Ampersand) => {
                    self.advance();
                    let last = match commands.pop() {
                        Some(val) => val,
                        None => {
                            return Err(ShellError::Parse("Syntax Error".to_string()));
                        }
                    };
                    commands.push(AstNode::Background(Box::new(last)));
                }
                _ => break,
            }
        }

        if commands.is_empty() {
            Ok(None)
        } else if commands.len() == 1 {
            let commande = match commands.into_iter().next() {
                Some(val) => val,
                None => return Err(ShellError::Parse("Syntax Error".to_string())),
            };
            Ok(Some(commande))
        } else {
            Ok(Some(AstNode::Sequence(commands)))
        }
    }
}
