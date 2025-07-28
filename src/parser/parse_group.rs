use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_group(&mut self) -> Result<Option<AstNode>, ShellError> {
        if !matches!(self.current(), Some(Token::OpenBrace)) {
            return Ok(None);
        }
        self.advance();

        while let Some(token) = self.current() {
            match token {
                Token::Newline | Token::Semicolon => {
                    self.advance();
                }
                _ => break,
            }
        }

        let mut commands = Vec::new();

        loop {
            match self.current() {
                Some(Token::CloseBrace) => break,
                None => return Err(ShellError::Parse("Unexpected EOF in command group".into())),
                _ => {}
            }

            if let Some(cmd) = self.parse_pipeline()? {
                commands.push(cmd);
            } else {
                return Err(ShellError::Parse("Expected command in group".into()));
            }

            match self.current() {
                Some(Token::Semicolon) | Some(Token::Newline) => {
                    self.advance();
                }
                Some(_) => {
                    return Err(ShellError::Parse(
                        "Expected `;`, `&`, or newline before `}`".into(),
                    ));
                }
                None => return Err(ShellError::Parse("Unexpected EOF".into())),
            }
        }

        self.advance();

        if commands.is_empty() {
            return Err(ShellError::Parse("Empty command group".into()));
        }

        let mut redirects: Vec<Redirect> = vec![];
        let mut current_pos = self.pos;

        while self.current().is_some() {
            match self.parse_redirection(current_pos) {
                Ok(Some((advance_by, redirect))) => {
                    redirects.push(redirect);
                    current_pos += advance_by;
                    self.pos = current_pos;
                    continue;
                }
                Ok(None) => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        return Ok(Some(AstNode::Group {
            commands,
            redirects,
        }));
    }
}
