use shell::commands::{cd, echo};
use shell::*;
pub mod config;
pub mod executer;
mod parser;
pub use parser::*;
use std::io::{self, Write};

fn main() {
    let mut buffer = String::new();
    print!("\x1B[2J\x1B[H"); //clear terminal
    loop {
        distplay_promt();
        io::stdout().flush().unwrap();
        buffer.clear();
        let _bytes_read = match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                println!("\nEOF");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from stdin: {}", e);
                break; // or continue / return depending on your case
            }
        };
        let cmd = parse(&buffer);
        executer::Execute(cmd);
    }
}
