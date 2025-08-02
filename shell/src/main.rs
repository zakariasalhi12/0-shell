pub use parser::Parser;
use shell::*;
pub mod config;
use colored::*;
use shell::envirement::ShellEnv;
use shell::exec::execute;
// pub mod executer;
// mod parser;
use config::*;
use shell::events_handler::{Shell, ShellMode};
use std::{env, io::{self, Write}};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = if let Some(pos) = args.iter().position(|arg| arg == "-c") {
        if let Some(cmd) = args.get(pos + 1) {
            ShellMode::Command(cmd.clone())
        } else {
            eprintln!("error: -c needs a command string");
            std::process::exit(1);
        }
    } else if atty::is(atty::Stream::Stdin) {
        ShellMode::Interactive
    } else {
        ShellMode::NonInteractive
    };

   
}
