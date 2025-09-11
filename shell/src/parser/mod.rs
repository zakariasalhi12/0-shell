pub mod parse_assignment;
pub mod parse_command_or_if;
pub mod parse_command;
pub mod parse_function;
pub mod parse_group;
pub mod parse_if;
pub mod parse_op;
pub mod parse_pipeline;
pub mod parse_redirection;
pub mod parse_sequence;
pub mod types;
pub mod parse_while_or_until;
pub mod parse_for;

use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::types::*;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn look_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    pub fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    pub fn is_eof(&self) -> bool {
        match self.current() {
            Some(Token::Eof) => true,
            None => true,
            _ => false,
        }
    }

    pub fn is_command_end(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Eof
                | Token::Newline
                | Token::Semicolon
                | Token::Ampersand
                | Token::LogicalAnd
                | Token::LogicalOr
                | Token::Pipe
        )
    }

    pub fn is_reserved_word(&self) -> bool {
        match self.current() {
            Some(Token::Word(word)) => {
                if word.parts.len() == 1 && word.quote == QuoteType::None {
                    match &word.parts[0] {
                        WordPart::Literal(part) => {
                            if (part.0 == "then" || part.0 == "fi" || part.0 == "else" || part.0 == "elif" || part.0 == "do" || part.0 == "done" || part.0 == "in" )
                                && part.1 == QuoteType::None
                            {
                                return true;
                            }
                            return false;
                        }
                        _ => {
                            return false;
                        }
                    }
                }
                return false;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn expect_word(&mut self, expected: &str) -> Result<(), ShellError> {
        match self.current() {
            Some(Token::Word(word)) => {
                if word.parts.len() == 1 {
                    if let WordPart::Literal(s) = &word.parts[0] {
                        if s.0 == expected && s.1 == QuoteType::None {
                            self.advance();
                            return Ok(());
                        }
                    }
                }
            }
            _ => {}
        }
        Err(ShellError::Syntax(format!("Expected '{}'", expected)))
    }

    pub fn expect_delimiter(&mut self) -> bool {
        match self.current() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::Ampersand) => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    pub fn parse(&mut self) -> Result<Option<AstNode>, ShellError> {
        match self.parse_sequence(false) {
            Ok(ast) => {
                if !self.is_eof() {
                    return Err(ShellError::Parse(format!(
                        "unexpected token at end of tokens{:#?}",
                        self.current()
                    )));
                }
                println!("{:?}", ast);
                return Ok(ast);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
