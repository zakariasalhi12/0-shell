use crate::error::ShellError;
use crate::lexer::types::QuoteType;
use crate::lexer::types::Token;
use crate::lexer::types::WordPart;
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
                Some(Token::Semicolon | Token::Newline) => {
                    match self.look_ahead(1){
                        Some(Token::Word(word)) =>{
                            if word.parts.len() == 1 && word.quote == QuoteType::None{
                                match &word.parts[0]{
                                    WordPart::Literal(part) =>{
                                        if (part.0 == "then" || part.0 == "fi" || part.0 == "else") && part.1 == QuoteType::None{
                                            break;
                                        }
                                    }
                                    _ =>{
                                        self.advance();
                                    }
                                }
                            }
                        },
                        _ =>{
                            self.advance();
                        }
                    }
                    // self.advance();
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
