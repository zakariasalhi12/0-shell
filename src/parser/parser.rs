use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, Word, WordPart};
use crate::parser::types::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
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

    pub fn remaining(&self) -> usize {
        self.tokens.len().saturating_sub(self.pos)
    }

    pub fn parse(&mut self) -> Result<Option<AstNode>, ShellError> {
        self.parse_command()
    }

    fn try_parse_assignment_at(&self, pos: usize) -> Option<(usize, (String, Vec<WordPart>))> {
        let token = self.tokens.get(pos)?;
        if let Token::Word(word) = token {
            if word.quote == QuoteType::None {
                if let Some(WordPart::Literal(part)) = word.parts.get(0) {
                    if let Some(eq_pos) = part.find('=') {
                        let key = part[..eq_pos].to_string();
                        if eq_pos == part.len() - 1 && word.parts.len() == 1 {
                            let next_token = self.tokens.get(pos + 1)?;
                            if let Token::Word(val) = next_token {
                                return Some((2, (key, val.parts.clone())));
                            } else {
                                return None;
                            }
                        }
                        let mut value_parts = Vec::new();
                        let after_eq = &part[eq_pos + 1..];
                        if !after_eq.is_empty() {
                            value_parts.push(WordPart::Literal(after_eq.to_string()));
                        }
                        value_parts.extend_from_slice(&word.parts[1..]);
                        return Some((1, (key, value_parts)));
                    }
                }
            }
        }
        None
    }

    fn try_parse_redirection_at(&self, pos: usize) -> Result<Option<(usize, Redirect)>, ShellError> {
        let current_token = self.tokens.get(pos).ok_or_else(|| {
            ShellError::Parse(
                "Unexpected end of input while parsing redirection".into(),
            )
        })?;

        match current_token {
            Token::RedirectOut => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '>'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target: target.clone(),
                        kind : RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '>'".into(),
                    ))
                }
            }
            Token::RedirectIn => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '<'".into())
                })?;
                if let Token::Word(target) = target_token {
                   
                    let redirect = Redirect {
                        fd: None,
                        target : target.clone(),
                        kind : RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '<'".into(),
                    ))
                }
            }
            Token::RedirectAppend => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '>>'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target : target.clone(),
                        kind : RedirectOp::Append,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '>>'".into(),
                    ))
                }
            }
            Token::RedirectHereDoc => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected delimiter after '<<'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target : target.clone(),
                        kind: RedirectOp::HereDoc,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected word token as heredoc delimiter after '<<'".into(),
                    ))
                }
            }
            Token::RedirectOutFd(fd_num) => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '>...'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: Some(*fd_num),
                        target: target.clone(),
                        kind: RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '>...'".into(),
                    ))
                }
            }
            Token::RedirectInFd(fd_num) => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '<...'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: Some(*fd_num),
                        target: target.clone(),
                        kind : RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '<...'".into(),
                    ))
                }
            }
            Token::RedirectAppendFd(fd_num) => {
                let target_token = self.tokens.get(pos + 1).ok_or_else(|| {
                    ShellError::Parse("Expected target after '>>...'".into())
                })?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: Some(*fd_num),
                        target: target.clone(),
                        kind : RedirectOp::Append,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename or '-' after redirection operator '>>...'".into(),
                    ))
                }
            }
            _ => {
                Ok(None)
            }
        }
    }

    pub fn parse_command(&mut self) -> Result<Option<AstNode>, ShellError> {
        let mut assignments = Vec::new();
        let mut current_pos = self.pos;

        loop {
            match self.try_parse_assignment_at(current_pos) {
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
            _ => return Ok(None),
        };

        let mut args = Vec::new();
        let mut redirects = Vec::new();
        let mut current_pos = self.pos;

        while self.current().is_some() {
            match self.try_parse_redirection_at(current_pos) {
                Ok(Some((advance_by, redirect))) => {
                    redirects.push(redirect);
                    current_pos += advance_by;
                    self.pos = current_pos;
                    continue;
                }
                Ok(None) => {
                }
                Err(e) => {
                    return Err(e)
                }
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