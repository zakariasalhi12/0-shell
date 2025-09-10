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
    InvalidInput(String),
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
            Self::InvalidInput(_) => 1,
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
        let format_error = |category: &str, color: colored::Color, msg: &str| {
            format!("{} {}", category.color(color).bold(), msg)
        };

        match self {
            ShellError::InvalidVariableSyntax => {
                write!(f, "{}", format_error("[Syntax]", colored::Color::Red, "Invalid variable syntax"))
            }
            ShellError::Io(err) => {
                write!(f, "{}", format_error("[IO]", colored::Color::Red, &err.to_string()))
            }
            ShellError::Syntax(msg) => {
                write!(f, "{}", format_error("[Syntax]", colored::Color::Red, msg))
            }
            ShellError::Parse(msg) => {
                write!(f, "{}", format_error("[Parse]", colored::Color::Red, msg))
            }
            ShellError::Eval(msg) => {
                write!(f, "{}", format_error("[Eval]", colored::Color::Yellow, msg))
            }
            ShellError::Exec(msg) => {
                write!(f, "{}", format_error("[Exec]", colored::Color::Magenta, msg))
            }
            ShellError::Expansion(msg) => {
                write!(f, "{}", format_error("[Expansion]", colored::Color::Cyan, msg))
            }
            ShellError::UnexpectedEof => {
                write!(f, "{}", format_error("[Syntax]", colored::Color::Red, "Unexpected end of file"))
            }
            ShellError::UnclosedQuote => {
                write!(f, "{}", format_error("[Syntax]", colored::Color::Red, "Unclosed quote"))
            }
            ShellError::InvalidVariable(var) => {
                write!(f, "{}", format_error("[Variable]", colored::Color::Blue, var))
            }
            ShellError::DivisionByZero => {
                write!(f, "{}", format_error("[Math]", colored::Color::Yellow, "Division by zero"))
            }
            ShellError::InvalidInput(err) => {
                write!(f, "{}", format_error("[Input]", colored::Color::Red, err))
            }
        }
    }
}
