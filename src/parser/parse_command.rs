use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, Word};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_command(&mut self) -> Result<Option<AstNode>, ShellError> {
        if let Some(func) = self.parse_function()? {
            return Ok(Some(func));
        }

        if let Ok(group) = self.parse_group() {
            if let Some(_) = group {
                return Ok(group);
            }
        }

        let mut assignments = Vec::new();
        let mut current_pos = self.pos;

        loop {
            match self.parse_assignment(current_pos) {
                Some((advance_by, assignment)) => {
                    assignments.push(assignment);
                    current_pos += advance_by;
                }
                None => break,
            }
        }
        self.pos = current_pos;

        let cmd_word = match self.current() {
            Some(Token::Word(word)) => {
                let word = (*word).clone();
                self.advance();
                word
            }
            _ => {
                // if there is no command its an assignement command !!!! So command will be None. In execution if we Command that have None cmd field we check assignement and assign them to the shell enviroment not just in command context
                if !assignments.is_empty() {
                    Word {
                        parts: vec![],
                        quote: QuoteType::None,
                    }
                } else {
                    return Ok(None);
                }
            }
        };

        let mut args = Vec::new();
        let mut redirects = Vec::new();
        let mut current_pos = self.pos;

        while self.current().is_some() {
            match self.parse_redirection(current_pos) {
                Ok(Some((advance_by, redirect))) => {
                    redirects.push(redirect);
                    current_pos += advance_by;
                    self.pos = current_pos;
                    continue;
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }

            if let Some(Token::Word(word)) = self.current() {
                args.push((*word).clone());
                self.advance();
                current_pos = self.pos;
            } else {
                break;
            }
        }

        Ok(Some(AstNode::Command {
            cmd: cmd_word,
            assignments,
            args,
            redirects,
        }))
    }
}
