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

    pub fn parse(&mut self) -> Result<AstNode, ShellError> {
        self.parse_command()
            .ok_or_else(|| ShellError::Parse("Unexpected token or end of input".into()))
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

    fn parse_redirect_target(word: &Word) -> RedirectTarget {
        if let Some(WordPart::Literal(part)) = word.parts.get(0) {
            if part.starts_with('&') {
                let fd_str = &part[1..];
                if fd_str == "-" {
                    return RedirectTarget::Close;
                }
                if let Ok(fd_num) = fd_str.parse::<u64>() {
                    return RedirectTarget::Fd(fd_num);
                }
            }
        }
        RedirectTarget::File(word.clone())
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
                if let Token::Word(target_word) = target_token {
                    let target = Self::parse_redirect_target(target_word);
                    let (kind, final_target) = match target {
                        RedirectTarget::Fd(_) | RedirectTarget::Close => {
                            (RedirectOp::DupWrite, target)
                        }
                        RedirectTarget::File(_) => (RedirectOp::Write, target),
                    };
                    let redirect = Redirect {
                        fd: None,
                        target: final_target,
                        kind,
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
                if let Token::Word(target_word) = target_token {
                    let target = Self::parse_redirect_target(target_word);
                    let (kind, final_target) = match target {
                        RedirectTarget::Fd(_) | RedirectTarget::Close => {
                            (RedirectOp::DupRead, target)
                        }
                        RedirectTarget::File(_) => (RedirectOp::Read, target),
                    };
                    let redirect = Redirect {
                        fd: None,
                        target: final_target,
                        kind,
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
                if let Token::Word(target_word) = target_token {
                    let target = Self::parse_redirect_target(target_word);
                    let kind = RedirectOp::Append;
                    let redirect = Redirect {
                        fd: None,
                        target,
                        kind,
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
                if let Token::Word(target_word) = target_token {
                    let target = RedirectTarget::File(target_word.clone());
                    let redirect = Redirect {
                        fd: None,
                        target,
                        kind: RedirectOp::HereDoc,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected word token as heredoc delimiter after '<<'".into(),
                    ))
                }
            }
            Token::Word(word) if word.quote == QuoteType::None => {
                if let Some(WordPart::Literal(part)) = word.parts.get(0) {
                    if let Ok(fd_num) = part.parse::<u64>() {
                        let operator_token = self.tokens.get(pos + 1).ok_or_else(|| {
                            ShellError::Parse(format!(
                                "Expected redirection operator after file descriptor '{}'",
                                fd_num
                            ))
                        })?;
                        match operator_token {
                            Token::RedirectOut => {
                                let target_token = self.tokens.get(pos + 2).ok_or_else(|| {
                                    ShellError::Parse("Expected target after '>...'".into())
                                })?;
                                if let Token::Word(target_word) = target_token {
                                    let target = Self::parse_redirect_target(target_word);
                                    let (kind, final_target) = match target {
                                        RedirectTarget::Fd(_) | RedirectTarget::Close => {
                                            (RedirectOp::DupWrite, target)
                                        }
                                        RedirectTarget::File(_) => (RedirectOp::Write, target),
                                    };
                                    let redirect = Redirect {
                                        fd: Some(fd_num),
                                        target: final_target,
                                        kind,
                                    };
                                    Ok(Some((3, redirect)))
                                } else {
                                     Err(ShellError::Parse(
                                        "Expected filename, file descriptor, or '-' after redirection operator '>&' or '>...'".into(),
                                    ))
                                }
                            }
                            Token::RedirectIn => {
                                let target_token = self.tokens.get(pos + 2).ok_or_else(|| {
                                    ShellError::Parse("Expected target after '<...'".into())
                                })?;
                                if let Token::Word(target_word) = target_token {
                                    let target = Self::parse_redirect_target(target_word);
                                    let (kind, final_target) = match target {
                                        RedirectTarget::Fd(_) | RedirectTarget::Close => {
                                            (RedirectOp::DupRead, target)
                                        }
                                        RedirectTarget::File(_) => (RedirectOp::Read, target),
                                    };
                                    let redirect = Redirect {
                                        fd: Some(fd_num),
                                        target: final_target,
                                        kind,
                                    };
                                    Ok(Some((3, redirect)))
                                } else {
                                     Err(ShellError::Parse(
                                        "Expected filename, file descriptor, or '-' after redirection operator '<&' or '<...'".into(),
                                    ))
                                }
                            }
                            Token::RedirectAppend => {
                                let target_token = self.tokens.get(pos + 2).ok_or_else(|| {
                                    ShellError::Parse("Expected target after '>>...'".into())
                                })?;
                                if let Token::Word(target_word) = target_token {
                                    let target = Self::parse_redirect_target(target_word);
                                    let redirect = Redirect {
                                        fd: Some(fd_num),
                                        target,
                                        kind: RedirectOp::Append,
                                    };
                                    Ok(Some((3, redirect)))
                                } else {
                                     Err(ShellError::Parse(
                                        "Expected filename, file descriptor, or '-' after redirection operator '>>&' or '>>...'".into(),
                                    ))
                                }
                            }
                            _ => {
                                Ok(None)
                            }
                        }
                    } else {
                         Ok(None)
                    }
                } else {
                     Ok(None)
                }
            }
            _ => {
                Ok(None)
            }
        }
    }

    pub fn parse_command(&mut self) -> Option<AstNode> {
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
            _ if !assignments.is_empty() => return None,
            _ => return None,
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
                Err(_) => {
                    return None;
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

        Some(AstNode::Command {
            cmd: cmd_word,
            assignments,
            args,
            redirects,
        })
    }
}