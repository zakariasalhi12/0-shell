use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, Word, WordPart};
use crate::parser::types::*;
use std::iter::Peekable;
use crate::lexer::tokenize::Tokenizer;
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
            if word.quote == QuoteType::None && word.parts.len() == 1 {
                if let WordPart::Literal(part) = &word.parts[0] {
                    if let Some(eq_pos) = part.find('=') {
                        let key = part[..eq_pos].to_string();

                        let val_str = &part[eq_pos + 1..];

                        self.tokens.next();

                        let mut tokenizer = Tokenizer::new(val_str);
                        match tokenizer.tokenize() {
                            Ok(tokens) => {
                                if let Some(Token::Word(val_word)) = tokens.iter().find(|t| matches!(t, Token::Word(_))) {
                                    return Some((key, val_word.parts.clone()));
                                } else {
                                    return Some((key, vec![WordPart::Literal(val_str.to_string())]));
                                }
                            }
                            Err(_) => {
                                return Some((key, vec![WordPart::Literal(val_str.to_string())]));
                            }
                        }
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
