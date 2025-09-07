use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_while_or_until(&mut self) -> Result<Option<AstNode>, ShellError> {
        // Check for 'while' or 'until'
        let word = match self.current() {
            Some(Token::Word(word)) => word,
            _ => return Ok(None),
        };

        if word.parts.len() != 1 {
            return Ok(None);
        }

        let loop_type = if let WordPart::Literal(s) = &word.parts[0] {
            match s.0.as_str() {
                "while" => "while",
                "until" => "until",
                _ => return Ok(None),
            }
        } else {
            return Ok(None);
        };

        self.advance();

        // Parse condition sequence
        let condition = match self.parse_sequence(true)? {
            Some(cmd) => cmd,
            None => {
                return Err(ShellError::Parse(format!(
                    "Expected command after '{}'",
                    loop_type
                )));
            }
        };

        self.expect_word("do")?;

        // Parse body sequence
        let body = match self.parse_sequence(true)? {
            Some(cmd) => cmd,
            None => {
                return Err(ShellError::Parse(format!(
                    "Expected body after 'do' in {} loop",
                    loop_type
                )));
            }
        };

        self.expect_word("done")?;

        Ok(Some(match loop_type {
            "while" => AstNode::While {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            "until" => AstNode::Until {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            _ => unreachable!(),
        }))
    }
}
