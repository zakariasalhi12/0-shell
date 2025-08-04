use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, Word, WordPart};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_function(&mut self) -> Result<Option<AstNode>, ShellError> {
        let start_pos = self.pos;
        let name = match self.current() {
            Some(Token::Word(word)) => word.clone(),
            _ => return Ok(None),
        };

        self.advance();

        if !matches!(self.current(), Some(Token::OpenParen)) {
            self.pos = start_pos;
            return Ok(None);
        }
        self.advance();

        if !matches!(self.current(), Some(Token::CloseParen)) {
            self.pos = start_pos;
            return Ok(None);
        }
        self.advance();

        let body = match self.current() {
            Some(Token::OpenBrace) => match self.parse_group()? {
                Some(body) => body,
                None => return Err(ShellError::Parse("Empty function body1".into())),
            },
            Some(Token::Word(word)) => {
                if let Some(WordPart::Literal(content)) = word.parts.get(0) {
                    if content.0.starts_with('{') {
                        let remaining = &content.0[1..];
                        if !remaining.is_empty() {
                            let remaining_word = Word {
                                parts: vec![WordPart::Literal((remaining.to_string(), QuoteType::None))],
                                quote : word.quote
                            };
                            self.tokens[self.pos] = Token::OpenBrace;
                            self.tokens
                                .insert(self.pos + 1, Token::Word(remaining_word));
                        }
                        match self.parse_group()? {
                            Some(body) => body,
                            None => {
                                return Err(ShellError::Parse("Empty function body".into()));
                            }
                        }
                    } else {
                        self.pos = start_pos;
                        return Ok(None);
                    }
                } else {
                    self.pos = start_pos;
                    return Ok(None);
                }
            }
            _ => {
                self.pos = start_pos;
                return Ok(None);
            }
        };

        Ok(Some(AstNode::FunctionDef {
            name: name.clone(),
            body: Box::new(body),
        }))
    }
}
