pub mod events_handler;
pub mod parser;
pub use parser::*;
pub mod envirement;
pub mod exec;
pub mod redirection;
pub mod commands {
    pub mod bg;
    pub mod cd;
    pub mod cp;
    pub mod echo;
    pub mod exit;
    pub mod export;
    pub mod fals;
    pub mod fg;
    pub mod jobs;
    pub mod kill;
    pub mod mkdir;
    pub mod mv;
    pub mod pwd;
    pub mod rm;
    pub mod test;
    pub mod tru;
    pub mod typ;
}
use crate::{error::ShellError, events_handler::OutputTarget};
use envirement as v;
use std::path::PathBuf;

pub mod shell_interactions {
    pub mod buffer;
    pub mod history_handler;
    pub mod rerender;
    pub mod utils;
}
pub mod features {
    pub mod history;
    pub mod jobs;
}

pub mod error;
pub mod eval;
pub mod executorr;
pub mod executor;
pub mod expansion;
pub mod lexer;

pub trait ShellCommand {
    fn execute(&self, env: &mut v::ShellEnv) -> Result<i32, ShellError>;
}
