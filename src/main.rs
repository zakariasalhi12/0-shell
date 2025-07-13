use shell::commands::{cd, echo};
use shell::*;
pub mod config;
use shell::{features::history, *};
// pub mod executer;
mod parser;
use config::*;
pub use parser::*;
use std::env;
use std::io::{self, Write};

fn main() {
    let mut buffer = String::new();
    {
        let map = ENV.lock().unwrap();

        for (k, v) in map.iter() {
            println!("{} : {}", k, v);
        }
    }

    let mut history = history::History::new();

    print!("\x1B[2J\x1B[H"); //clear terminal
    loop {
        distplay_promt();
        io::stdout().flush().unwrap();
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        // let args: Vec<String> = buffer
        //     .trim()
        //     .split(" ")
        //     .map(|str| str.to_string())
        //     .collect();
        // let mut vece: Vec<String> = vec![];
        // vece.push("src".to_string());
        // cd::Cd::new(vece).execute();
        
        // history.run(&mut buffer);
        history.save(buffer.to_owned());
        // let cmd = parse(&buffer);
        // exec::execute(cmd);
    }
}
