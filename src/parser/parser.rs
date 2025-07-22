use crate::error::ShellError;
use crate::lexer::tokenize::Tokenizer;
use crate::lexer::types::{QuoteType, Token, Word, WordPart};
use crate::parser::types::*;
use std::iter::Peekable;

pub struct Parser<'a> {
    tokens: Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    // pub fn check(&mut self, expected: &Token) -> bool {
    //     match self.tokens.peek() {
    //         Some(tok) => *tok == expected,
    //         None => false,
    //     }
    // }

    pub fn parse(&mut self) -> Result<AstNode, ShellError> {
        self.parse_command()
            .ok_or_else(|| ShellError::Parse("Unexpected end of input".into()))
    }

    fn parse_word(&mut self) -> Option<Word> {
        match self.tokens.peek() {
            Some(Token::Word(word)) => Some((*word).clone()),
            _ => None,
        }
    }

    fn parse_assignment(&mut self) -> Option<(String, Vec<WordPart>)> {
        let token = self.tokens.peek()?;
        if let Token::Word(word) = token {
            if word.quote == QuoteType::None{
                if let Some(WordPart::Literal(part)) = word.parts.get(0) {
                    if let Some(pos) = part.find('=') {
                        let key = part[..pos].to_string();
                        if pos == part.len() - 1 && word.parts.len() == 1 {
                            self.tokens.next();

                            if let Some(Token::Word(val)) = self.tokens.peek() {
                                self.tokens.next();
                                return Some((key, val.parts.clone()));
                            }
                        }
                        let mut value_parts = Vec::new();

                        let after_eq = &part[pos + 1..];
                        if !after_eq.is_empty() {
                            value_parts.push(WordPart::Literal(after_eq.to_string()));
                        }

                        value_parts.extend_from_slice(&word.parts[1..]);

                        self.tokens.next();
                        return Some((key, value_parts));
                    }
                }
            }
        }
        None
    }

    pub fn parse_command(&mut self) -> Option<AstNode> {
        let mut assignments = Vec::new();

        while let Some((key, val_parts)) = self.parse_assignment() {
            assignments.push((key, val_parts));
        }

        let cmd_word = self.parse_word()?;
        self.tokens.next();

        let mut args = Vec::new();
        while let Some(Token::Word(word)) = self.tokens.peek() {
            args.push((*word).clone());
            self.tokens.next();
        }

        Some(AstNode::Command {
            cmd: cmd_word,
            assignments,
            args,
            redirects: vec![], // TODO: implement redirects
        })
    }
}
