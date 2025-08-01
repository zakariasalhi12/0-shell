use crate::lexer::types::{QuoteType, Token, WordPart};
use crate::parser::Parser;

impl Parser {
    pub fn parse_assignment(&self, pos: usize) -> Option<(usize, (String, Vec<WordPart>))> {
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
}
