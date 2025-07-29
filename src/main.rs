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
    // let mut buffer = String::new();
    let mut env = ShellEnv::new();
    // {
    //     let map = ENV.lock().unwrap();

    //     for (k, v) in map.iter() {
    //         println!("{} : {}", k, v);
    //     }
    // }
}

// pub fn Parse_input(buffer: &str) {
//     match lexer::tokenize::Tokenizer::new(buffer.trim().to_owned().as_str()).tokenize() {
//         Ok(res) => {
//             // println!("{}", "== Tokens ==".bold().bright_blue());
//             // for token in &res {
//             //     println!("{:#?}", token);
//             // }
//             match Parser::new(res).parse() {
//                 Ok(ast) => {
//                     println!("{}", "== AST Output ==".bold().bright_yellow());
//                     match ast {
//                         Some(ast) => {
//                             println!("{}", ast); // Optionally keep for debugging
//                             let exit_code = execute(&ast, &mut envirement).unwrap_or(1);
//                             println!("[exit code: {}]", exit_code);
//                         }
//                         None => println!("empty AST"),
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("{}", "== AST Parse Error ==".bold().red());
//                     eprintln!("{:#?}", e);
//                 }
//             }
//         }

//         Err(err) => {
//             eprintln!("{}", "== Tokenization Error ==".bold().red());
//             eprintln!("{:#?}", err);
//         }
//     }
// }
