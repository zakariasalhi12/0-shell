use colored::Colorize;
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
            ShellError::Io(err) => {
                write!(f, "{}", format!("IO error: {}", err).red().bold().italic())
            }
            ShellError::Syntax(msg) => {
                write!(
                    f,
                    "{}",
                    format!("Syntax error: {}", msg).red().bold().italic()
                )
            }
            ShellError::Parse(msg) => write!(
                f,
                "{}",
                format!("Parse error: {}", msg).red().bold().italic()
            ),
            ShellError::Eval(msg) => {
                write!(
                    f,
                    "{}",
                    format!("Evaluation error: {}", msg).red().bold().italic()
                )
            }
            ShellError::Exec(msg) => {
                write!(
                    f,
                    "{}",
                    format!("Execution error: {}", msg).red().bold().italic()
                )
            }
            ShellError::Expansion(msg) => {
                write!(
                    f,
                    "{}",
                    format!("Expansion error: {}", msg).red().bold().italic()
                )
            }
            ShellError::UnexpectedEof => {
                write!(
                    f,
                    "{}",
                    String::from("Unexpected end of file").red().bold().italic()
                )
            }
            ShellError::UnclosedQuote => {
                write!(
                    f,
                    "{}",
                    String::from("Unclosed quote").red().bold().italic()
                )
            }
            ShellError::InvalidVariable(var) => {
                write!(
                    f,
                    "{}",
                    format!("Invalid variable: {}", var).red().bold().italic()
                )
            }
            ShellError::DivisionByZero => {
                write!(
                    f,
                    "{}",
                    String::from("Division by zero").red().bold().italic()
                )
            }
        }
    }
}
