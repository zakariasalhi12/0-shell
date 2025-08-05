// pub use parser::Parser;
pub use shell::parser;
pub mod config;
use shell::events_handler::{self, ShellMode};

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

    events_handler::Shell::new(mode).run();
}
