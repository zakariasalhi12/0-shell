use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ShellError {
    InvalidVariableSyntax,
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

impl ShellError {
    pub fn code(&self) -> i32 {
        match self {
            ShellError::InvalidVariableSyntax => 2,
            ShellError::Io(_) => 1,
            ShellError::Syntax(_) => 2,
            ShellError::Parse(_) => 2,
            ShellError::Eval(_) => 3,
            ShellError::Exec(_) => 126,
            ShellError::Expansion(_) => 4,
            ShellError::UnexpectedEof => 2,
            ShellError::UnclosedQuote => 2,
            ShellError::InvalidVariable(_) => 5,
            ShellError::DivisionByZero => 6,
        }
    }
}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::InvalidVariableSyntax => write!(f, "Invalid variable syntax"),
            ShellError::Io(err) => write!(f, "IO error: {}", err),
            ShellError::Syntax(msg) => write!(f, "Syntax error: {}", msg),
            ShellError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ShellError::Eval(msg) => write!(f, "Evaluation error: {}", msg),
            ShellError::Exec(msg) => write!(f, "Execution error: {}", msg),
            ShellError::Expansion(msg) => write!(f, "Expansion error: {}", msg),
            ShellError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ShellError::UnclosedQuote => write!(f, "Unclosed quote"),
            ShellError::InvalidVariable(var) => write!(f, "Invalid variable: {}", var),
            ShellError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}
