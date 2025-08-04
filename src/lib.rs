pub mod config;
pub mod shell;
pub mod parser;
pub use parser::*;
pub mod executer;
pub mod commands {
    pub mod cat;
    pub mod cd;
    pub mod cp;
    pub mod echo;
    pub mod export;
    pub mod ls;
    pub mod mkdir;
    pub mod mv;
    pub mod pwd;
    pub mod rm;
}
use std::env;
use std::io::{Stdout};
use std::path::PathBuf;
use termion::raw::RawTerminal;

use crate::shell_interactions::utils::print_out;

pub mod shell_interactions {
    pub mod edit_buffer;
    pub mod pop_from_buffer;
    pub mod utils;
    pub mod history_handler;
    pub mod positions_handler;
}
pub mod features {
    pub mod history;
}
// pub mod config;
enum Colors {
    // WHITE(String),
    // GREY(String),
    // BLUE(String),
    YELLOW(String),
}

impl Colors {
    fn to_ansi(&self) -> String {
        match self {
            // Colors::WHITE(text) => format!("\x1b[1;37m{}\x1b[0m", text),
            // Colors::GREY(text) => format!("\x1b[1;30m{}\x1b[0m", text),
            // Colors::BLUE(text) => format!("\x1b[1;34m{}\x1b[0m", text),
            Colors::YELLOW(text) => format!("\x1b[1;31m{}\x1b[0m", text),
        }
    }
}

pub fn get_first_element<'a>(s: &'a str, pattern: &str) -> &'a str {
    s.split(pattern).next().unwrap_or("")
}

pub fn get_current_directory() -> Result<String, String> {
    match env::current_dir() {
        Ok(path) => match path.file_name() {
            Some(name) => Ok(name.to_string_lossy().to_string()),
            None => Ok("/".to_string()),
        },
        Err(e) => match redirect_to_home() {
            Ok(val) => Ok(val),
            Err(e) => Err(e.to_string()),
        },
    }
}

pub fn display_prompt(stdout: &mut Option<RawTerminal<Stdout>>) {
    let current_directory: String = get_current_directory().unwrap();
    let prompt = Colors::YELLOW(format!("➜ {} ", current_directory));
    print_out(stdout, &format!("{}", prompt.to_ansi()));
}

pub fn prompt_len() -> usize {
    let current_directory: String = get_current_directory().unwrap();
    format!("➜ {} ", current_directory).chars().count()
}

pub fn redirect_to_home() -> std::io::Result<String> {
    let home = env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    env::set_current_dir(&home)?;

    let dir_name = home
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("/")
        .to_string();

    Ok(dir_name)
}

pub trait ShellCommand {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
