use crate::error::ShellError;
pub use crate::lexer::types::{QuoteType, State, Token, Word, WordPart};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Tokenizer<'a> {
    pub chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ShellError> {
        let mut tokens = Vec::new();
        let mut state = State::Default;
        let mut buffer = String::new();
        let mut parts: Vec<WordPart> = vec![];

        while let Some(&c) = self.chars.peek() {
            match (&mut state, c) {
                (State::Default, ' ' | '\t' | '\n') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: QuoteType::None,
                        }));
                        parts.clear();
                    }
                    if c == '\n'{
                        tokens.push(Token::Newline);
                    }
                    state = State::Default;
                }
                (State::InDoubleQuote | State::Default | State::InWord, '$') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if let Some(c) = self.chars.peek() {
                        match *c {
                            '{' => {
                                self.chars.next();
                                let mut var = String::new();
                                self.read_until_matching("{", "}", &mut var)?;
                                parts.push(WordPart::VariableSubstitution(var));
                            }
                            '(' => {
                                self.chars.next();
                                if let Some('(') = self.chars.peek() {
                                    self.chars.next();
                                    let mut expr = String::new();
                                    self.read_until_matching("((", "))", &mut expr)?;
                                    parts.push(WordPart::ArithmeticSubstitution(expr));
                                } else {
                                    let mut cmd = String::new();
                                    self.read_until_matching("(", ")", &mut cmd)?;
                                    parts.push(WordPart::CommandSubstitution(cmd));
                                }
                            }
                            c if c.is_alphanumeric() || c == '_' => {
                                let mut var = String::new();
                                while let Some(&ch) = self.chars.peek() {
                                    if ch.is_alphanumeric() || ch == '_' {
                                        var.push(ch);
                                        self.chars.next();
                                    } else {
                                        break;
                                    }
                                }
                                parts.push(WordPart::VariableSubstitution(var));
                            }
                            _ => buffer.push('$'),
                        }
                    } else {
                        buffer.push('$');
                    }
                }

                (State::Default, '{') => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c) if c.is_whitespace() || *c == ';'|| *c == '|'|| *c == '&'|| *c == '\n' => {
                            tokens.push(Token::OpenBrace);
                            state = State::Default;
                        }
                        Some(_) => {
                            buffer.push('{');
                            state = State::InWord;
                        }
                        None => {
                            buffer.push('{');
                            state = State::InWord;
                        }
                    }
                }

                (State::Default, '}') => {
                    self.chars.next();
                    let next_is_delimiter = match self.chars.peek() {
                        Some(c) => c.is_whitespace() 
                            || *c == ';' 
                            || *c == '|' 
                            || *c == '&' 
                            || *c == '\n'
                            || *c == ')'  
                            || *c == '#'  
                            || *c == '(',
                        None => true,
                    };

                    if next_is_delimiter {
                        tokens.push(Token::CloseBrace);
                        state = State::Default;
                    } else {
                        buffer.push('}');
                        state = State::InWord;
                    }
                }

                (State::Default, '(') =>{
                    
                    self.chars.next();
                    tokens.push(Token::OpenParen);
                    state = State::Default;
                }

                (State::Default, ')') =>{
                    self.chars.next();
                    tokens.push(Token::CloseParen);
                    state = State::Default;
                }

                (State::InWord, '"') | (State::Default, '"') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: QuoteType::None,
                        }));
                        parts.clear();
                    }
                    state = State::InDoubleQuote;
                }
                (State::InWord, '\'') | (State::Default, '\'') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: QuoteType::None,
                        }));
                        parts.clear();
                    }
                    state = State::InSingleQuote;
                }
                (State::Default, '#') => {
                    self.chars.next();
                    while let Some(c) = self.chars.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                (State::Default, '&') => {
                    self.chars.next();
                    if let Some('&') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LogicalAnd);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }
                (State::Default, '|') => {
                    self.chars.next();
                    if let Some('|') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LogicalOr);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }
                (State::Default, '!') => {
                    self.chars.next();
                    tokens.push(Token::LogicalNot);
                }
                (State::Default, '>') => {
                    self.chars.next();
                    state = State::MaybeRedirectOut2;
                }
                (State::MaybeRedirectOut2, '&') => {
                    buffer.push('&');
                    self.chars.next();
                    tokens.push(Token::RedirectOut);
                    state = State::InWord;
                }
                (State::MaybeRedirectOut2, '>') => {
                    self.chars.next();
                    tokens.push(Token::RedirectAppend);
                    state = State::Default;
                }
                (State::MaybeRedirectOut2, _) => {
                    tokens.push(Token::RedirectOut);
                    state = State::Default;
                }
                (State::Default, '<') => {
                    self.chars.next();
                    state = State::MaybeRedirectIn2;
                }
                (State::MaybeRedirectIn2, '&') => {
                    buffer.push('&');
                    self.chars.next();
                    tokens.push(Token::RedirectIn);
                    state = State::InWord;
                }
                (State::MaybeRedirectIn2, '<') => {
                    self.chars.next();
                    tokens.push(Token::RedirectHereDoc);
                    state = State::Default;
                }
                (State::MaybeRedirectIn2, _) => {
                    tokens.push(Token::RedirectIn);
                    state = State::Default;
                }
                (State::Default, ';') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: QuoteType::None,
                        }));
                        parts.clear();
                    }
                    tokens.push(Token::Semicolon);
                    state = State::Default;
                }
                (State::Default, _) => {
                    self.chars.next();
                    buffer.push(c);
                    state = State::InWord;
                }
                (State::InWord, ' ' | '\t' | '\n' | '|' | ';' | '&' | '!' | '(' | ')' ) => {
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: QuoteType::None,
                        }));
                        parts.clear();
                    }
                    state = State::Default;
                }
                (State::InWord, '>') => {
                    self.chars.next();
                    if buffer.chars().all(|ch| ch.is_ascii_digit()) && !buffer.is_empty() {
                        match buffer.parse::<u64>() {
                            Ok(fd_num) => {
                                buffer.clear();
                                state = State::MaybeRedirectOut2Fd(fd_num);
                            }
                            Err(_) => {
                                parts.push(WordPart::Literal(buffer.clone()));
                                buffer.clear();
                                if !parts.is_empty() {
                                    tokens.push(Token::Word(Word {
                                        parts: parts.clone(),
                                        quote: QuoteType::None,
                                    }));
                                    parts.clear();
                                }
                                state = State::MaybeRedirectOut2;
                            }
                        }
                    } else {
                        if !buffer.is_empty() {
                            parts.push(WordPart::Literal(buffer.clone()));
                            buffer.clear();
                        }
                        if !parts.is_empty() {
                            tokens.push(Token::Word(Word {
                                parts: parts.clone(),
                                quote: QuoteType::None,
                            }));
                            parts.clear();
                        }
                        state = State::MaybeRedirectOut2;
                    }
                }
                (State::InWord, '<') => {
                    self.chars.next();
                    if buffer.chars().all(|ch| ch.is_ascii_digit()) && !buffer.is_empty() {
                        match buffer.parse::<u64>() {
                            Ok(fd_num) => {
                                buffer.clear();
                                state = State::MaybeRedirectIn2Fd(fd_num);
                            }
                            Err(_) => {
                                parts.push(WordPart::Literal(buffer.clone()));
                                buffer.clear();
                                if !parts.is_empty() {
                                    tokens.push(Token::Word(Word {
                                        parts: parts.clone(),
                                        quote: QuoteType::None,
                                    }));
                                    parts.clear();
                                }
                                state = State::MaybeRedirectIn2;
                            }
                        }
                    } else {
                        if !buffer.is_empty() {
                            parts.push(WordPart::Literal(buffer.clone()));
                            buffer.clear();
                        }
                        if !parts.is_empty() {
                            tokens.push(Token::Word(Word {
                                parts: parts.clone(),
                                quote: QuoteType::None,
                            }));
                            parts.clear();
                        }
                        state = State::MaybeRedirectIn2;
                    }
                }
                (State::InWord, c) => {
                    self.chars.next();
                    buffer.push(c);
                }
                (State::InDoubleQuote, '"') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    tokens.push(Token::Word(Word {
                        parts: parts.clone(),
                        quote: QuoteType::Double,
                    }));
                    parts.clear();
                    state = State::InWord;
                }
                (State::InDoubleQuote, '\\') => {
                    self.chars.next();
                    if let Some(next) = self.chars.next() {
                        buffer.push(next);
                    }
                }
                (State::InDoubleQuote, c) => {
                    self.chars.next();
                    buffer.push(c);
                }
                (State::InSingleQuote, '\'') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        parts.push(WordPart::Literal(buffer.clone()));
                        buffer.clear();
                    }
                    tokens.push(Token::Word(Word {
                        parts: parts.clone(),
                        quote: QuoteType::Single,
                    }));
                    parts.clear();
                    state = State::InWord;
                }
                (State::InSingleQuote, c) => {
                    self.chars.next();
                    buffer.push(c);
                }
                (State::MaybeRedirectOut2Fd(fd_num), '&') => {
                    buffer.push('&');
                    self.chars.next();
                    tokens.push(Token::RedirectOutFd(*fd_num));
                    state = State::InWord;
                }
                (State::MaybeRedirectOut2Fd(fd_num), '>') => {
                    self.chars.next();
                    tokens.push(Token::RedirectAppendFd(*fd_num));
                    state = State::Default;
                }
                (State::MaybeRedirectOut2Fd(fd_num), _) => {
                    tokens.push(Token::RedirectOutFd(*fd_num));
                    state = State::Default;
                }
                (State::MaybeRedirectIn2Fd(fd_num), '&') => {
                    buffer.push('&');
                    self.chars.next();
                    tokens.push(Token::RedirectInFd(*fd_num));
                    state = State::InWord;
                }
                (State::MaybeRedirectIn2Fd(fd_num), '<') => {
                    self.chars.next();
                    tokens.push(Token::RedirectHereDoc);
                    state = State::Default;
                }
                (State::MaybeRedirectIn2Fd(fd_num), _) => {
                    tokens.push(Token::RedirectInFd(*fd_num));
                    state = State::Default;
                }
            }
        }

        if !buffer.is_empty() {
            parts.push(WordPart::Literal(buffer));
        }
        if !parts.is_empty() {
            tokens.push(Token::Word(Word {
                parts,
                quote: QuoteType::None,
            }));
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn read_until_matching(
        &mut self,
        start: &str,
        end: &str,
        buffer: &mut String,
    ) -> Result<(), ShellError> {
        let start_len = start.len();
        let end_len = end.len();
        let mut depth = 1;
        while let Some(_) = self.chars.peek() {
            if self.peek_matches(start) {
                for _ in 0..start_len {
                    buffer.push(self.chars.next().unwrap());
                }
                depth += 1;
                continue;
            }
            if self.peek_matches(end) {
                for _ in 0..end_len {
                    self.chars.next();
                }
                depth -= 1;
                if depth == 0 {
                    break;
                }
                buffer.push_str(end);
                continue;
            }
            if let Some(c) = self.chars.next() {
                buffer.push(c);
            }
        }
        if depth == 0 {
            Ok(())
        } else {
            Err(ShellError::Syntax(format!("unclosed {}", start)))
        }
    }

    fn peek_matches(&mut self, s: &str) -> bool {
        let mut iter = self.chars.clone();
        for expected_char in s.chars() {
            match iter.next() {
                Some(c) if c == expected_char => (),
                _ => return false,
            }
        }
        true
    }
}
