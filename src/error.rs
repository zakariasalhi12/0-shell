use std::io;

#[derive(Debug)]
pub enum ShellError {
    Io(io::Error),
    Syntax(String),
    Parse(String),
    Eval(String),
    Exec(String),
    Expansion(String),
    UnexpectedEof,
    UnclosedQuote,
    InvalidVariable(String),
    DivisionByZero,
}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}
