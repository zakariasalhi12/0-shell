pub mod events_handler;
pub mod parser;
pub use parser::*;
pub mod envirement;
pub mod exec;
pub mod redirection;
pub mod commands {
    pub mod cd;
    pub mod cp;
    pub mod echo;
    pub mod export;
    pub mod mkdir;
    pub mod mv;
    pub mod pwd;
    pub mod rm;
    pub mod typ;
    pub mod exit;
}
use crate::events_handler::OutputTarget;
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
}

pub mod error;
pub mod eval;
pub mod expansion;
pub mod jobs;
pub mod lexer;



pub trait ShellCommand {
    fn execute(&self, env: &mut v::ShellEnv) -> std::io::Result<()>;
}
