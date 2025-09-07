use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
      pub fn parse_for(&mut self) -> Result<Option<AstNode>, ShellError> {
        // Check for `for`
        let word = match self.current() {
            Some(Token::Word(word)) => word,
            _ => return Ok(None),
        };

        if word.parts.len() != 1 || word.quote != QuoteType::None {
            return Ok(None);
        }

        if let WordPart::Literal(s) = &word.parts[0] {
            if s.0 != "for" {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        self.advance(); // consume `for`

        // Parse loop variable
        let var = match self.current() {
            Some(Token::Word(word)) if word.parts.len() == 1 && word.quote == QuoteType::None => {
                if let WordPart::Literal(s) = &word.parts[0] {
                    let tmp = s.0.clone();
                    self.advance();
                    tmp
                } else {
                    return Err(ShellError::Parse("Invalid loop variable".into()));
                }
            }
            _ => {
                return Err(ShellError::Parse("Expected loop variable after 'for'".into()));
            }
        };

        // Expect `in`
        self.expect_word("in")?;

        // Parse and expand the word list
        let mut values = Vec::new();
        while let Some(token) = self.current() {
            match token {
                Token::Word(word) => {
                    values.push(word.clone());
                    self.advance();
                }
                Token::Semicolon | Token::Newline => {
                    self.advance();
                        if self.is_reserved_word(){
                            break;
                        }else{
                            return Err(ShellError::Parse(String::from("expected words after in")));
                        }
                }
                _ =>{
                    return Err(ShellError::Parse(String::from("expected words after in")));
                    
                }
            }
        }

        // Expect `do`
        self.expect_word("do")?;

        // Parse body
        let body = match self.parse_sequence(true)? {
            Some(cmd) => cmd,
            None => {
                return Err(ShellError::Parse("Expected commands in 'do' body".into()));
            }
        };

        // Expect `done`
        self.expect_word("done")?;

        Ok(Some(AstNode::For {
            var,
            values,
            body: Box::new(body),
        }))
    }
}