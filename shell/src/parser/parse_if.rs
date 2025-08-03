use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_if(&mut self) -> Result<Option<AstNode>, ShellError>{
        loop{
            match self.current() {
                Some(Token::Word(word)) =>{
                    if word.parts.len() == 1 {
                        if let Some(WordPart::Literal(word)) = word.parts.get(0){
                            if word.0 == "if"{
                                self.advance();
                                
                            }else{
                                return Ok(None);
                            }
                        }
                    }
                },
                _ =>{

                },
                None =>{
                    break;
                }
            }
        }
        Ok(None)
    }
}