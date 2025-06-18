use std::io::{self, Write};
use shell::*;
use shell::commands::echo;

fn main() {
    let mut buffer = String::new();

    loop {
        distplay_promt();      
        io::stdout().flush().unwrap();
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        let args : Vec<String> = buffer.trim().split(" ").map(|str| str.to_string()).collect();

    }
}