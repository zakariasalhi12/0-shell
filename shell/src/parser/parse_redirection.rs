use crate::error::ShellError;
use crate::lexer::types::Token;
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_redirection(&self, pos: usize) -> Result<Option<(usize, Redirect)>, ShellError> {
        let current_token = self.tokens.get(pos).ok_or_else(|| {
            ShellError::Parse("Unexpected end of input while parsing redirection".into())
        })?;

        match current_token {
            Token::RedirectOut => {
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '>'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target: target.clone(),
                        kind: RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '>'"
                            .into(),
                    ))
                }
            }
            Token::RedirectIn => {
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '<'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target: target.clone(),
                        kind: RedirectOp::Write,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '<'"
                            .into(),
                    ))
                }
            }
            Token::RedirectAppend => {
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '>>'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target: target.clone(),
                        kind: RedirectOp::Append,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '>>'".into(),
                    ))
                }
            }
            Token::RedirectHereDoc => {
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected delimiter after '<<'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: None,
                        target: target.clone(),
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
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '>...'".into()))?;
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
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '<...'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: Some(*fd_num),
                        target: target.clone(),
                        kind: RedirectOp::Read,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename, file descriptor, or '-' after redirection operator '<...'".into(),
                    ))
                }
            }
            Token::RedirectAppendFd(fd_num) => {
                let target_token = self
                    .tokens
                    .get(pos + 1)
                    .ok_or_else(|| ShellError::Parse("Expected target after '>>...'".into()))?;
                if let Token::Word(target) = target_token {
                    let redirect = Redirect {
                        fd: Some(*fd_num),
                        target: target.clone(),
                        kind: RedirectOp::Append,
                    };
                    Ok(Some((2, redirect)))
                } else {
                    Err(ShellError::Parse(
                        "Expected filename or '-' after redirection operator '>>...'".into(),
                    ))
                }
            }
            _ => Ok(None),
        }
    }
}
