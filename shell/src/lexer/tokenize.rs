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
        let mut buffer = (String::new(), QuoteType::None);
        let mut parts: Vec<WordPart> = vec![];
        let mut where_im_at = QuoteType::None;

        while let Some(&c) = self.chars.peek() {
            match (&mut state, c) {
                (State::Default | State::InWord, '\\') => {
                    self.chars.next();
                    if let Some(next) = self.chars.next() {
                        match next {
                            '\\' => buffer.0.push('\\'),
                            ' ' => buffer.0.push(' '),
                            '$' => buffer.0.push('$'),
                            other => buffer.0.push(other),
                        }
                    } else {
                        buffer.0.push('\\');
                    }
                }

                (State::InDoubleQuote, '\\') => {
                    self.chars.next();
                    if let Some(next) = self.chars.next() {
                        match next {
                            '\\' => buffer.0.push('\\'),
                            '"' => buffer.0.push('"'),
                            other => {
                                buffer.0.push('\\');
                                buffer.0.push(other)
                            }
                        }
                    }
                }

                (State::InSingleQuote, '\\') => {
                    self.chars.next();
                    if let Some(next) = self.chars.next() {
                        buffer.0.push('\\');
                        buffer.0.push(next);
                    } else {
                        buffer.0.push('\\');
                    }
                }

                (State::Default, ' ' | '\t' | '\n') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: where_im_at,
                        }));
                        parts.clear();
                        where_im_at = QuoteType::None;
                    }
                    if c == '\n' {
                        tokens.push(Token::Newline);
                    }
                    state = State::Default;
                }

                (State::InDoubleQuote | State::Default | State::InWord, '$') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    if let Some(c) = self.chars.peek() {
                        match *c {
                            '{' => {
                                self.chars.next();
                                if !buffer.0.is_empty() {
                                    parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                                    buffer.0.clear();
                                }
                                let mut var = String::new();
                                self.read_until_matching("{", "}", &mut var)?;
                                parts.push(WordPart::VariableSubstitution(var));
                            }
                            '(' => {
                                self.chars.next();
                                if !buffer.0.is_empty() {
                                    parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                                    buffer.0.clear();
                                }
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
                            c if c.is_alphanumeric() || c == '_' || c == '?' => {
                                let mut var = String::new();
                                if c.is_ascii_digit() {
                                    var.push(c);
                                    self.chars.next();
                                    while let Some(&ch) = self.chars.peek() {
                                        if ch.is_ascii_digit() {
                                            var.push(ch);
                                            self.chars.next();
                                        } else {
                                            break;
                                        }
                                    }
                                } else {
                                    while let Some(&ch) = self.chars.peek() {
                                        if ch.is_alphanumeric() || ch == '_' || ch == '?' {
                                            var.push(ch);
                                            self.chars.next();
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                parts.push(WordPart::VariableSubstitution(var));
                            }
                            _ => buffer.0.push('$'),
                        }
                    } else {
                        buffer.0.push('$');
                    }
                }

                (State::Default, '~') => {
                    self.chars.next();
                    state = State::InWord;
                    parts.push(WordPart::VariableSubstitution(String::from("~")));
                }

                (State::Default, '{') => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c)
                            if c.is_whitespace()
                                || *c == ';'
                                || *c == '|'
                                || *c == '&'
                                || *c == '\n' =>
                        {
                            tokens.push(Token::OpenBrace);
                            state = State::Default;
                        }
                        Some(_) => {
                            buffer.0.push('{');
                            state = State::InWord;
                        }
                        None => {
                            buffer.0.push('{');
                            state = State::InWord;
                        }
                    }
                }

                (State::Default, '}') => {
                    self.chars.next();
                    let next_is_delimiter = match self.chars.peek() {
                        Some(c) => {
                            c.is_whitespace()
                                || *c == ';'
                                || *c == '|'
                                || *c == '&'
                                || *c == '\n'
                                || *c == ')'
                                || *c == '#'
                                || *c == '('
                        }
                        None => true,
                    };

                    if next_is_delimiter {
                        tokens.push(Token::CloseBrace);
                        state = State::Default;
                    } else {
                        buffer.0.push('}');
                        state = State::InWord;
                    }
                }

                (State::Default, '(') => {
                    self.chars.next();
                    tokens.push(Token::OpenParen);
                    state = State::Default;
                }

                (State::Default, ')') => {
                    self.chars.next();
                    tokens.push(Token::CloseParen);
                    state = State::Default;
                }
                (State::Default, '"') => {
                    self.chars.next();
                    buffer.1 = QuoteType::Double;
                    state = State::InDoubleQuote;
                    if where_im_at == QuoteType::None {
                        where_im_at = QuoteType::Double
                    }
                }

                (State::InWord, '"') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    state = State::InDoubleQuote;
                }

                (State::Default, '\'') => {
                    self.chars.next();
                    state = State::InSingleQuote;
                    if where_im_at == QuoteType::None {
                        where_im_at = QuoteType::Double
                    }
                }
                (State::InWord, '\'') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
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
                    match self.chars.peek() {
                        Some(' ') | Some('\t') | Some('\n') => {
                            tokens.push(Token::LogicalNot);
                        }
                        _ => {
                            buffer.1 = QuoteType::None;
                            buffer.0.push('!');
                            state = State::InWord;
                        }
                    }
                }
                (State::Default, '>') => {
                    self.chars.next();
                    state = State::MaybeRedirectOut2;
                }
                (State::MaybeRedirectOut2, '&') => {
                    buffer.0.push('&');
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
                    buffer.0.push('&');
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
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: where_im_at,
                        }));
                        where_im_at = QuoteType::None;
                        parts.clear();
                    }
                    tokens.push(Token::Semicolon);
                    state = State::Default;
                }
                (State::Default, _) => {
                    self.chars.next();
                    buffer.0.push(c);
                    state = State::InWord;
                }
                (State::InWord, ' ' | '\t' | '\n' | '|' | ';' | '&' | '!' | '(' | ')') => {
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    if !parts.is_empty() {
                        tokens.push(Token::Word(Word {
                            parts: parts.clone(),
                            quote: where_im_at,
                        }));
                        where_im_at = QuoteType::None;
                        parts.clear();
                    }
                    state = State::Default;
                }
                (State::InWord, '>') => {
                    self.chars.next();
                    if !buffer.0.is_empty() && buffer.0.chars().all(|ch| ch.is_ascii_digit()) {
                        match buffer.0.parse::<u64>() {
                            Ok(fd_num) => {
                                buffer.0.clear();
                                state = State::MaybeRedirectOut2Fd(fd_num);
                            }
                            Err(_) => {
                                parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                                buffer.0.clear();
                                if !parts.is_empty() {
                                    tokens.push(Token::Word(Word {
                                        parts: parts.clone(),
                                        quote: where_im_at,
                                    }));
                                    parts.clear();
                                    where_im_at = QuoteType::None;
                                }
                                state = State::MaybeRedirectOut2;
                            }
                        }
                    } else {
                        if !buffer.0.is_empty() {
                            parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                            buffer.0.clear();
                        }
                        if !parts.is_empty() {
                            tokens.push(Token::Word(Word {
                                parts: parts.clone(),
                                quote: where_im_at,
                            }));
                            parts.clear();
                            where_im_at = QuoteType::None;
                        }
                        state = State::MaybeRedirectOut2;
                    }
                }
                (State::InWord, '<') => {
                    self.chars.next();
                    if buffer.0.chars().all(|ch| ch.is_ascii_digit()) && !buffer.0.is_empty() {
                        match buffer.0.parse::<u64>() {
                            Ok(fd_num) => {
                                buffer.0.clear();
                                state = State::MaybeRedirectIn2Fd(fd_num);
                            }
                            Err(_) => {
                                parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                                buffer.0.clear();
                                if !parts.is_empty() {
                                    tokens.push(Token::Word(Word {
                                        parts: parts.clone(),
                                        quote: where_im_at,
                                    }));
                                    parts.clear();
                                    where_im_at = QuoteType::None;
                                }
                                state = State::MaybeRedirectIn2;
                            }
                        }
                    } else {
                        if !buffer.0.is_empty() {
                            parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                            buffer.0.clear();
                        }
                        if !parts.is_empty() {
                            tokens.push(Token::Word(Word {
                                parts: parts.clone(),
                                quote: where_im_at,
                            }));
                            parts.clear();
                            where_im_at = QuoteType::None;
                        }
                        state = State::MaybeRedirectIn2;
                    }
                }
                (State::InWord, c) => {
                    self.chars.next();
                    buffer.0.push(c);
                    buffer.1 = QuoteType::None;
                }
                (State::InDoubleQuote, '"') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    buffer.1 = QuoteType::None;
                    state = State::InWord;
                }

                (State::InDoubleQuote, c) => {
                    self.chars.next();
                    buffer.0.push(c);
                }
                (State::InSingleQuote, '\'') => {
                    self.chars.next();
                    if !buffer.0.is_empty() {
                        parts.push(WordPart::Literal((buffer.0.clone(), buffer.1)));
                        buffer.0.clear();
                    }
                    state = State::InWord;
                }
                (State::InSingleQuote, c) => {
                    self.chars.next();
                    buffer.0.push(c);
                }
                (State::MaybeRedirectOut2Fd(fd_num), '&') => {
                    buffer.0.push('&');
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
                    buffer.0.push('&');
                    self.chars.next();
                    tokens.push(Token::RedirectInFd(*fd_num));
                    state = State::InWord;
                }
                (State::MaybeRedirectIn2Fd(_), '<') => {
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

        if !buffer.0.is_empty() {
            parts.push(WordPart::Literal((buffer.0, buffer.1)));
        }
        if !parts.is_empty() {
            tokens.push(Token::Word(Word {
                parts,
                quote: where_im_at,
            }));
        }

        if state == State::InDoubleQuote {
            return Err(ShellError::Syntax("missing quote: \"".to_string()));
        }
        if state == State::InSingleQuote {
            return Err(ShellError::Syntax("missing quote: '".to_string()));
        }

        match state {
            State::MaybeRedirectOut2 => {
                tokens.push(Token::RedirectOut);
            }
            State::MaybeRedirectIn2 => {
                tokens.push(Token::RedirectIn);
            }
            State::MaybeRedirectOut2Fd(fd) => {
                tokens.push(Token::RedirectOutFd(fd));
            }
            State::MaybeRedirectIn2Fd(fd) => {
                tokens.push(Token::RedirectInFd(fd));
            }
            _ => {}
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
                    let char = match self.chars.next() {
                        Some(val) => val,
                        None => return Err(ShellError::UnexpectedEof),
                    };
                    buffer.push(char);
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
