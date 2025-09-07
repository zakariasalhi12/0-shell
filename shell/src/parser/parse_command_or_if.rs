use crate::Parser;
use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::types::AstNode;

impl Parser {
    pub fn parse_command_or_if(&mut self) -> Result<Option<AstNode>, ShellError> {
        let should_negate = match self.current() {
            Some(Token::LogicalNot) => {
                self.advance();
                true
            }
            _ => false,
        };

        let node = if let Some(if_node) = self.parse_if()? {
            if_node
        }else if let Some(while_node) = self.parse_while_or_until()? {
            while_node
        }else if let Some(for_node) = self.parse_for()? {
            for_node
        }else {
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
        return Ok(Some(node));
    }
}
