pub mod config;
pub mod events_handler;
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
use std::io::{Stdout, Write};
use termion::raw::RawTerminal;

use crate::events_handler::print_out;
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
        Err(e) => Err(e.to_string()),
    }
}

pub fn display_promt(stdout: &mut Option<RawTerminal<Stdout>>) {
    let current_directory: String = get_current_directory().unwrap();
    let prompt = Colors::YELLOW(format!("➜ {} ", current_directory));
    print_out(stdout, &format!("{}", prompt.to_ansi()));
    // match stdout {
    //     Some(raw_stdout) => {
    //         write!(raw_stdout, "{}", prompt.to_ansi()).unwrap();
    //         raw_stdout.flush().unwrap();
    //     }
    //     None => {
    //         let mut std = std::io::stdout();
    //         write!(std, "{}", prompt.to_ansi()).unwrap();
    //         std.flush().unwrap();
    //     }
    // }
}

pub fn promt_len() -> usize {
    let current_directory: String = get_current_directory().unwrap();
    format!("➜ {} ", current_directory).chars().count()
}

pub trait ShellCommand {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
