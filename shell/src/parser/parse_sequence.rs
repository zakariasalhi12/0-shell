use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_sequence(&mut self, in_if_condition: bool) -> Result<Option<AstNode>, ShellError> {
        let mut commands = Vec::new();

        loop {
            if let Some(cmd) = self.parse_pipeline()? {
                commands.push(cmd);
            } else {
                break;
            }

            match self.current() {
                Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::Ampersand) => {
                    let is_background = matches!(self.current(), Some(Token::Ampersand));
                    self.advance();

                    if is_background {
                        let last = commands
                            .pop()
                            .ok_or_else(|| ShellError::Parse("Syntax Error".to_string()))?;
                        commands.push(AstNode::Background(Box::new(last)));
                    }

                    if in_if_condition && self.is_reserved_word() {
                        break;
                    }
                }
                _ => {
                    if in_if_condition {
                        return Err(ShellError::Parse(
                            "expected sequence to end with Newline, Semicolon, or &".to_string(),
                        ));
                    } else {
                        break;
                    }
                }
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
