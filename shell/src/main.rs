use std::io::{self, Write};


enum Colors {
    White(String),
    Black(String),
    Blue(String),
    Red(String),
}

impl Colors {
    fn to_ansi(&self) -> String {
        match self {
            Colors::White(text) => format!("\x1b[37m{}\x1b[0m", text),
            Colors::Black(text) => format!("\x1b[30m{}\x1b[0m", text),
            Colors::Blue(text) => format!("\x1b[34m{}\x1b[0m", text),
            Colors::Red(text) => format!("\x1b[31m{}\x1b[0m", text),
        }
    }
}

fn main() {
    let mut buffer = String::new();

    loop {
        let prompt = Colors::Blue(String::from("➜ 0-shell :"));
        print!("{}" , prompt.to_ansi());
        
        io::stdout().flush().unwrap();
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        
        let input = buffer.trim();
        match input {
            "exit" => break,
            "clear" => print!("\x1B[2J\x1B[H"),
            _ => println!("0-shell command not found: {}" , input),
        }
    }
}