use shell::commands::{cd, echo};
use shell::*;
pub mod executer;
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
        let cmd = parse(&buffer);
        executer::Execute(cmd);
    }
}
