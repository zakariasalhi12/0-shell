pub mod parse_assignment;
pub mod parse_command;
pub mod parse_function;
pub mod parse_group;
pub mod parse_op;
pub mod parse_pipeline;
pub mod parse_redirection;
pub mod parse_sequence;
pub mod parse_if;
pub mod types;

use crate::error::ShellError;
use crate::lexer::types::Token;
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

    pub fn parse(&mut self) -> Result<Option<AstNode>, ShellError> {
        let ast = self.parse_sequence();
        if !self.is_eof() {
            return Err(ShellError::Parse(format!(
                "unexpected token at end of tokens{:#?}",
                self.current()
            )));
        }
        return ast;
    }
}
