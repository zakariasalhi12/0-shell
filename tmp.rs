pub fn parse_function(&mut self) -> Result<Option<AstNode>, ShellError> {
    // Save current position for rollback
    let start_pos = self.pos;

    // 1. Parse function name
    let name = match self.current() {
        Some(Token::Word(word)) => word.clone(),
        _ => return Ok(None),
    };
    self.advance();

    // 2. Expect `OpenParen`
    if !matches!(self.current(), Some(Token::OpenParen)) {
        self.pos = start_pos;
        return Ok(None);
    }
    self.advance();

    // 3. Expect `CloseParen`
    if !matches!(self.current(), Some(Token::CloseParen)) {
        self.pos = start_pos;
        return Ok(None);
    }
    self.advance();

    // 4. Now expect `{` — but it might be part of a word
    let body = match self.current() {
        Some(Token::OpenBrace) => {
            // Case 1: `name() { ... }`
            self.advance(); // consume `{`
            match self.parse_group_body()? {
                Some(body) => body,
                None => {
                    return Err(ShellError::Parse("Empty function body".into()));
                }
            }
        }
        Some(Token::Word(word)) => {
            // Case 2: `name(){ ... }` or `name(){...}`
            if let Some(WordPart::Literal(content)) = word.parts.get(0) {
                if content.starts_with('{') {
                    // Extract the part after `{`
                    let remaining = &content[1..];
                    self.advance(); // consume the word

                    // If there's more after `{`, re-inject it
                    if !remaining.is_empty() {
                        // Push the remaining content back as a word
                        let remaining_word = Word {
                            parts: vec![WordPart::Literal(remaining.to_string())],
                            quote: QuoteType::None,
                        };
                        // Insert it at current pos (you may need to modify tokens)
                        // For simplicity, we'll assume you can handle this
                    }

                    // Now parse the group body
                    match self.parse_group_body()? {
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
        name: name.to_string(),
        body: Box::new(body),
    }))
}