use crate::error::ShellError;
use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::Parser;
use crate::parser::types::*;

impl Parser {
    pub fn parse_if(&mut self) -> Result<Option<AstNode>, ShellError> {
        // Check for `if`
        let word = match self.current() {
            Some(Token::Word(word)) => word,
            _ => return Ok(None),
        };

        if word.parts.len() != 1 {
            return Ok(None);
        }

        if let WordPart::Literal(s) = &word.parts[0] {
            if s.0 != "if" || s.1 != QuoteType::None {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        self.advance();

        let condition = match self.parse_sequence(true)? {
            Some(cmd) => cmd,
            None => {
                return Err(ShellError::Syntax("Expected command after 'if'".into()));
            }
        };

        self.expect_word("then")?;

        // Parse `then` branch
        let then_branch = match self.parse_sequence(true)? {
            Some(cmd) => cmd,
            None => {
                return Err(ShellError::Syntax(
                    "Expected body after 'then' branch".into(),
                ));
            }
        };

        let mut elif: Vec<(Box<AstNode>, Box<AstNode>)> = Vec::new();
        let mut else_branch: Option<Box<AstNode>> = None;

        // Parse `elif` and `else`
        while let Some(Token::Word(word)) = self.current() {
            if let Some(WordPart::Literal(s)) = word.parts.get(0) {
                if s.0 == "elif" && s.1 == QuoteType::None {
                    self.advance();
                    if let Some(elif_condition) = self.parse_sequence(true)? {
                        self.expect_word("then")?;
                        if let Some(elif_then) = self.parse_sequence(true)? {
                            elif.push((Box::new(elif_condition), Box::new(elif_then)));
                        } else {
                            return Err(ShellError::Parse(String::from(
                                "Expected body after elif then",
                            )));
                        }
                    } else {
                        return Err(ShellError::Parse(String::from(
                            "expected condition after elif",
                        )));
                    }
                } else if s.0 == "else" && s.1 == QuoteType::None {
                    self.advance();
                    if let Some(else_body) = self.parse_sequence(true)? {
                        else_branch = Some(Box::new(else_body));
                    } else {
                        return Err(ShellError::Parse(String::from("expected body after else")));
                    }

                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Expect `fi`
        self.expect_word("fi")?;

        Ok(Some(AstNode::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            elif,
            else_branch,
        }))
    }
}
