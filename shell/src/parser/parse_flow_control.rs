use crate::Parser;
use crate::error::ShellError;
use crate::lexer::types::{Token, WordPart};
use crate::types::AstNode;

impl Parser {
    pub fn parse_flow_control(&mut self) -> Result<Option<AstNode>, ShellError> {
        let word = match self.current() {
            Some(Token::Word(word)) => word,
            _ => return Ok(None),
        };

        if word.parts.len() != 1 {
            return Ok(None);
        }

        let cmd = match &word.parts[0] {
            WordPart::Literal(s) if s.0 == "break" => "break",
            WordPart::Literal(s) if s.0 == "continue" => "continue",
            _ => return Ok(None),
        };

        self.advance();

        let level_word = match self.current() {
            Some(Token::Word(word)) => {
                Some((*word).clone())
            }
            _ => None,
        };

        if level_word.is_some() {
            self.advance();
        }

        let node = match cmd {
            "break" => AstNode::Break(level_word),
            "continue" => AstNode::Continue(level_word),
            _ => unreachable!(),
        };

        Ok(Some(node))
    }
}