use crate::Parser;
use crate::envirement::ShellEnv;
use crate::exec::execute;
use crate::lexer::tokenize::Tokenizer;
use crate::{display_promt, prompt_len};
use std::io::*;
use termion::raw::RawTerminal;
use termion::{
    clear,
    cursor::{self, Up},
};

pub fn calc_termlines_in_buffer(buffer_size: usize) -> u16 {
    let (width, _) = termion::terminal_size().unwrap_or((80, 24));
    let prompt_length = prompt_len() as u16;
    let total_content = prompt_length + buffer_size as u16;
    (total_content + width - 1) / width
}

pub fn print_out(w: &mut Option<RawTerminal<Stdout>>, input: &str) {
    match w {
        Some(raw_stdout) => {
            match write!(raw_stdout, "{}", input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            match raw_stdout.flush() {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
        }
        None => {
            let mut std = std::io::stdout();
            match write!(std, "{}", input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            match std.flush() {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
        }
    }
}

pub fn clear_terminal(stdout: &mut Option<RawTerminal<Stdout>>, buffer: &mut String) {
    buffer.clear();
    print_out(stdout, &format!("{}{}\r", clear::All, cursor::Goto(1, 1)));
    display_promt(stdout);
}

pub fn clear_current_line(stdout: &mut Option<RawTerminal<Stdout>>) {
    print_out(stdout, &format!("{}\r", clear::CurrentLine));
}

pub fn clear_buff_ter(stdout: &mut Option<RawTerminal<Stdout>>, bufer: String) {
    let lines = calc_termlines_in_buffer(bufer.len());
    // println!("{}", lines);
    for _i in 0..lines - 1 {
        print_out(stdout, &format!("{}\r", Up(1)));
        clear_current_line(stdout);
    }
}

pub fn parse_input(buffer: &str, mut env: &mut ShellEnv) {
    match Tokenizer::new(buffer.trim().to_owned().as_str()).tokenize() {
        Ok(res) => match Parser::new(res).parse() {
            Ok(ast) => match ast {
                Some(ast) => {
                    match execute(&ast, &mut env) {
                        Ok(_status) => {
                            print!("\r");
                        }
                        Err(e) => {
                            env.set_last_status(e.code());
                        },
                    }
                }
                None => println!("empty AST"),
            },
            Err(e) => {
                eprintln!("{:#?}", e);
            }
        },

        Err(err) => {
            eprintln!("{:#?}", err);
        }
    }
}
