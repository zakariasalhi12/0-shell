use crate::Parser;
use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::types::AstNode;

impl Parser {
    pub fn parse_background(&mut self) -> Result<Option<AstNode>, ShellError> {
        let should_negate = match self.current() {
            Some(Token::LogicalNot) => {
                self.advance();
                true
            }
            _ => false,
        };

        let node = if let Some(if_node) = self.parse_if()? {
            if_node
        } else {
            match self.parse_command()? {
                Some(cmd) => {
                    if should_negate {
                        AstNode::Not(Box::new(cmd))
                    } else {
                        cmd
                    }
                }
                None => return Ok(None),
            }
        };

        if matches!(self.current(), Some(Token::Ampersand)) {
            if !Parser::is_reserved_word(self.look_ahead(1)) {
                self.advance();
                if let Some(Token::Semicolon) = self.current() {
                    return Err(ShellError::Parse(
                        "Unexpected `;` after background `&`".into(),
                    ))
                }
            }
            return Ok(Some(AstNode::Background(Box::new(node))));
        }
        return Ok(Some(node));
    }
}
