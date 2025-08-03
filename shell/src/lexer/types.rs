use std::process::Command;
use std::process::Stdio;
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub parts: Vec<WordPart>,
    pub quote: QuoteType,
}

use crate::envirement::ShellEnv;

impl Word {
    pub fn expand(&self, env: &ShellEnv) -> String {
        let mut result = String::new();
        for part in &self.parts {
            match (part, self.quote) {
                (
                    WordPart::CommandSubstitution(expression),
                    QuoteType::Double | QuoteType::None,
                ) => {
                    // if let Some(shell_path) = env.get("0") {
                    // println!("shell path: {}", shell_path);
                    let command = Command::new("./bin/0shell")
                        .arg("-c")
                        .arg(expression)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .unwrap();
                    let output = match command.wait_with_output() {
                        Ok(output) => output,
                        Err(e) => {
                            eprintln!("Error running command: {}", e);
                            return "".to_string();
                        }
                    };

                    let output_str = match String::from_utf8(output.stdout) {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("Error converting output to string: {}", e);
                            return "".to_string();
                        }
                    };
                    result.push_str(&output_str.trim());

                    // }
                }
                (WordPart::CommandSubstitution(word), QuoteType::Single) => {
                    result.push_str(&word);
                }
                (WordPart::VariableSubstitution(var), QuoteType::Double | QuoteType::None) => {
                    if let Some(value) = env.get(&var) {
                        result.push_str(&value);
                    }
                }
                (WordPart::VariableSubstitution(word), QuoteType::Single) => {
                    result.push_str(&word);
                }
                (
                    WordPart::ArithmeticSubstitution(expression),
                    QuoteType::Double | QuoteType::None,
                ) => {}
                (WordPart::ArithmeticSubstitution(word), QuoteType::Single) => {
                    result.push_str(&word);
                }
                (WordPart::Literal(word), _) => {
                    result.push_str(&word);
                }
            }
        }
        return result;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(Word),
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
    RedirectInFd(u64),
    RedirectOutFd(u64),
    RedirectAppendFd(u64),
    RedirectHereDoc,
    Semicolon,
    Ampersand,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum QuoteType {
    Single,
    Double,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WordPart {
    Literal(String),
    VariableSubstitution(String),   // $USER
    ArithmeticSubstitution(String), // $((1 + 2))
    CommandSubstitution(String),    // $(whoami)
}

#[derive(Debug)]
pub enum State {
    Default,
    InWord,
    InDoubleQuote,
    InSingleQuote,
    MaybeRedirectOut2,
    MaybeRedirectIn2,
    MaybeRedirectOut2Fd(u64),
    MaybeRedirectIn2Fd(u64),
}
