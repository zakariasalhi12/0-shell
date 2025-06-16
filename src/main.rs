use std::io::{self, Write};

use shell::*;


fn main() {
    let mut buffer = String::new();

    loop {
        distplay_promt();      
        io::stdout().flush().unwrap();
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        
        let input = buffer.trim();
        match input {
            "exit" => break,
            "clear" => print!("\x1B[2J\x1B[H"),
            _ => println!("0-shell command not found: {}" , get_first_element(input , " ")),
        }
    }
}