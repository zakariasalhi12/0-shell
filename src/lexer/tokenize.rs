use crate::error::ShellError;
use crate::lexer::types::*;

pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens: Vec<Token> = vec![];
    let mut chars = input.chars().peekable();
    let mut in_dbl_quotes = false;
    let mut in_sgl_quotes = false;
    let mut buff = String::new();
    while let Some(c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '\n' => {
                tokens.push(Token::Newline);
            }
            '&' => {
                chars.next();
                if let Some(c) = chars.peek() {
                    if *c == '&' {
                        chars.next();
                        tokens.push(Token::AndIf);
                    } else {
                        tokens.push(Token::Background);
                    }
                }
            }
            '|' => {
                chars.next();
                if let Some(c) = chars.peek() {
                    if *c == '|' {
                        chars.next();
                        tokens.push(Token::Pipe);
                    } else {
                        tokens.push(Token::OrIf);
                    }
                }
            }
            ';' => {
                chars.next();
                tokens.push(Token::Background);
            }
            '$' => {
                chars.next();
                if let Some(c) = chars.peek() {
                    match c {
                        '(' => {
                            chars.next();
                            if let Some(c) = chars.peek() {
                                match c {
                                    '(' => {
                                        chars.next();
                                        if in_dbl_quotes{
                                            let mut parts: Vec<WordPart> = vec![];
                                            
                                        }else{

                                        }
                                    }
                                    _ => {
                                        todo!()
                                    }
                                }
                            }
                        },
                        _ => {
                            todo!()
                        }
                    }
                }
            }
            '"' => {
                if in_dbl_quotes {
                    buff.push(*c);
                }
                if !in_sgl_quotes {
                    in_dbl_quotes = !in_dbl_quotes;
                }
            }
            '\'' => {
                todo!()
            }
            _ => {}
        }
    }
    Ok(tokens)
}
