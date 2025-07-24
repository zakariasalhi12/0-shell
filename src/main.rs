use shell::commands::{cd, echo};
use shell::parser::parser::Parser;
use shell::*;
pub mod config;
use colored::*;
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
        println!("loop");
        distplay_promt();
        io::stdout().flush().unwrap();
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();

        println!("{}", buffer);

        match lexer::tokenize::Tokenizer::new(buffer.to_owned().as_str()).tokenize() {
            Ok(res) => {
                println!("{}", "== Tokens ==".bold().bright_blue());
                for token in &res {
                    println!("{:#?}", token);
                }

                println!("{}", "== Parsing AST ==".bold().bright_green());
                match Parser::new(res).parse() {
                    Ok(ast) => {
                        println!("{}", "== AST Output ==".bold().bright_yellow());
                        println!("{:#?}", ast);
                    }
                    Err(e) => {
                        eprintln!("{}", "== AST Parse Error ==".bold().red());
                        eprintln!("{:#?}", e);
                    }
                }
            }

            Err(err) => {
                eprintln!("{}", "== Tokenization Error ==".bold().red());
                eprintln!("{:#?}", err);
            }
        }

        // let args: Vec<String> = buffer
        //     .trim()
        //     .split(" ")
        //     .map(|str| str.to_string())
        //     .collect();
        // let mut vece: Vec<String> = vec![];
        // vece.push("src".to_string());
        // cd::Cd::new(vece).execute();

        // history.run(&mut buffer);
        // history.save(buffer.to_owned());
        // let cmd = parse(&buffer);
        // exec::execute(cmd);
    }
}
