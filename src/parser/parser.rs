use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, Word, WordPart};
use crate::parser::types::*;
use std::iter::Peekable;
use std::process::Command;

pub struct Parser<'a> {
    tokens: Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        return Self {
            tokens: tokens.iter().peekable(),
        };
    }

    pub fn check(&mut self, expected: &Token) -> bool {
        if let Some(tok) = self.tokens.peek() {
            return *tok == expected;
        }
        return false;
    }

    pub fn parse(&mut self) -> Result<AstNode, ShellError> {
        while let Some(tok) = self.tokens.peek() {
            //    self.parse_command()
        }
        Err(ShellError::Parse("unimplemented parser".into()))
    }

    fn parse_word(&mut self) -> Option<Word> {
        match self.tokens.peek()? {
            Token::Word(word) => Some(word.to_owned()),
            _ => None,
        }
    }

    fn parse_assignement(&mut self) -> Option<(String, String)> {
        while let Some(Token::Word(word)) = self.tokens.peek() {
            if word.quote == QuoteType::None && word.parts.len() == 1 {
                match &word.parts[0] {
                    WordPart::Literal(part) => {
                        let mut seen_equal = 0;
                        let mut key = String::new();
                        let mut val = String::new();
                        for c in part.chars() {
                            if c == '=' {
                                seen_equal += 1;
                            } else if seen_equal == 0 {
                                key.push(c);
                            } else if seen_equal == 1 {
                                val.push(c);
                            } else if seen_equal > 1 {
                                return None;
                            }
                        }
                        return Some((key, val));
                    }
                    _ => return None,
                }
            }
        }
        None
    }

    pub fn parse_command(&mut self) -> Option<AstNode> {
        let mut assignments = vec![];

        while let Some((key, val)) = self.parse_assignement() {
            assignments.push((key, val));
            self.tokens.next(); 
        }

        let cmd = self.parse_word()?;
        self.tokens.next(); // consume it

        let mut args: Vec<Word> = vec![];

        // parse remaining words as arguments
        while let Some(Token::Word(word)) = self.tokens.peek() {
            args.push(word.clone());
            self.tokens.next();
        }

        Some(AstNode::Command {
            cmd,
            assignments,
            args,
            redirects: vec![]
        })
    }
}
