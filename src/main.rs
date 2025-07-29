pub use parser::Parser;
use shell::*;
pub mod config;
use colored::*;
use shell::envirement::ShellEnv;
use shell::exec::execute;
// pub mod executer;
// mod parser;
use config::*;
use shell::events_handler::Shell;
use std::io::{self, Write};
fn main() {
    let mut shell = match Shell::new() {
        Ok(val) => val,
        Err(_) => return,
    };
    shell.run();
}
