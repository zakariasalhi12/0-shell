use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    RedirectHereDoc,
    Semicolon,
    Ampersand,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    Eof,
}

#[derive(Debug)]
enum State {
    Default,
    InWord,
    InDoubleQuote,
    InSingleQuote,
    MaybeRedirectOut2,
    MaybeRedirectIn2,
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut state = State::Default;
        let mut buffer = String::new();

        while let Some(&c) = self.chars.peek() {
            match (&mut state, c) {
                // --- Whitespace ---
                (State::Default, ' ' | '\t' | '\n') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        tokens.push(Token::Word(buffer.clone()));
                        buffer.clear();
                    }
                    state = State::Default;
                }

                // --- Double Quote ---
                (State::Default, '"') => {
                    self.chars.next();
                    state = State::InDoubleQuote;
                }

                // --- Single Quote ---
                (State::Default, '\'') => {
                    self.chars.next();
                    state = State::InSingleQuote;
                }

                // --- Comments ---
                (State::Default, '#') => {
                    self.chars.next();
                    while let Some(c) = self.chars.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                    if !buffer.is_empty() {
                        tokens.push(Token::Word(buffer.clone()));
                        buffer.clear();
                    }
                    state = State::Default;
                }

                // --- Logical Operators ---
                (State::Default, '&') => {
                    self.chars.next();
                    if let Some('&') = self.chars.peek().copied() {
                        self.chars.next();
                        tokens.push(Token::LogicalAnd);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }

                (State::Default, '|') => {
                    self.chars.next();
                    if let Some('|') = self.chars.peek().copied() {
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

                // --- Redirections ---
                (State::Default, '>') => {
                    self.chars.next();
                    state = State::MaybeRedirectOut2;
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

                (State::MaybeRedirectIn2, '<') => {
                    self.chars.next();
                    tokens.push(Token::RedirectHereDoc);
                    state = State::Default;
                }

                (State::MaybeRedirectIn2, _) => {
                    tokens.push(Token::RedirectIn);
                    state = State::Default;
                }

                // --- Semicolon ---
                (State::Default, ';') => {
                    self.chars.next();
                    if !buffer.is_empty() {
                        tokens.push(Token::Word(buffer.clone()));
                        buffer.clear();
                    }
                    tokens.push(Token::Semicolon);
                    state = State::Default;
                }

                // --- Start of Word ---
                (State::Default, _) => {
                    self.chars.next();
                    buffer.push(c);
                    state = State::InWord;
                }

                // --- In Word: End of word on special char ---
                (State::InWord, ' ' | '\t' | '\n' | '|' | '>' | '<' | ';' | '&' | '!') => {
                    tokens.push(Token::Word(buffer.clone()));
                    buffer.clear();
                    state = State::Default;
                }

                (State::InWord, c) => {
                    self.chars.next();
                    buffer.push(c);

                    // Handle special syntax inside words
                    if buffer.ends_with("${") {
                        buffer.truncate(buffer.len() - 2);
                        buffer.push_str("${");
                        self.read_until_matching('{', '}', &mut buffer);
                    } else if buffer.ends_with("$((\")") {
                        buffer.truncate(buffer.len() - 4);
                        buffer.push_str("$((\"");
                        self.read_until_matching('(', ')', &mut buffer);
                    } else if buffer.ends_with("$(( ") {
                        buffer.truncate(buffer.len() - 4);
                        buffer.push_str("$(( ");
                        self.read_until_matching('(', ')', &mut buffer);
                    } else if buffer.ends_with("$(") {
                        buffer.truncate(buffer.len() - 2);
                        buffer.push_str("$(");
                        self.read_until_matching('(', ')', &mut buffer);
                    }
                }

                // --- Inside Double Quote ---
                (State::InDoubleQuote, '"') => {
                    self.chars.next();
                    tokens.push(Token::Word(buffer.clone()));
                    buffer.clear();
                    state = State::Default;
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

                    // Handle special syntax inside double quotes
                    if buffer.ends_with("${") {
                        buffer.truncate(buffer.len() - 2);
                        buffer.push_str("${");
                        self.read_until_matching('{', '}', &mut buffer);
                    } else if buffer.ends_with("$((\")") {
                        buffer.truncate(buffer.len() - 4);
                        buffer.push_str("$((\"");
                        self.read_until_matching('(', ')', &mut buffer);
                    } else if buffer.ends_with("$(") {
                        buffer.truncate(buffer.len() - 2);
                        buffer.push_str("$(");
                        self.read_until_matching('(', ')', &mut buffer);
                    }
                }

                // --- Inside Single Quote ---
                (State::InSingleQuote, '\'') => {
                    self.chars.next();
                    tokens.push(Token::Word(buffer.clone()));
                    buffer.clear();
                    state = State::Default;
                }

                (State::InSingleQuote, c) => {
                    self.chars.next();
                    buffer.push(c);
                }
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            tokens.push(Token::Word(buffer));
        }

        tokens
    }

    /// Reads until matching closing delimiter, supporting nesting
    fn read_until_matching(&mut self, open: char, close: char, buffer: &mut String) {
        let mut depth = 1;

        while let Some(c) = self.chars.next() {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
            }

            buffer.push(c);

            if depth == 0 {
                break;
            }
        }
    }
}
