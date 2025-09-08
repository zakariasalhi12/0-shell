use crate::Parser;
use crate::envirement::ShellEnv;
use crate::exec::execute;
use crate::lexer::tokenize::Tokenizer;
use std::io::*;
use termion::raw::RawTerminal;
use termion::{
    clear,
    cursor::{self, Up},
};
use unicode_width::UnicodeWidthStr;

use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

impl CursorPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

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

pub fn clear_buff_ter(stdout: &mut Option<RawTerminal<Stdout>>, buffer: String) {
    let lines = calc_termlines_in_buffer(UnicodeWidthStr::width(buffer.as_str()));
    for _i in 0..lines - 1 {
        print_out(stdout, &format!("{}\r", Up(1)));
        clear_current_line(stdout);
    }
}

pub fn parse_input(buffer: &str, mut env: &mut ShellEnv) {
    match Tokenizer::new(buffer.to_owned().as_str()).tokenize() {
        Ok(res) => match Parser::new(res).parse() {
            Ok(ast) => match ast {
                Some(ast) => match execute(&ast, &mut env) {
                    Ok(_status) => {
                        print!("\r");
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        env.set_last_status(e.code());
                    }
                },
                None => {}
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        },

        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

pub fn display_promt(stdout: &mut Option<RawTerminal<std::io::Stdout>>) {
    let current_directory: String = match get_current_directory() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let prompt = Colors::YELLOW(format!("➜ {} ", current_directory));
    print_out(stdout, &format!("{}", prompt.to_ansi()))
}

pub fn prompt_len() -> usize {
    let current_directory: String = match get_current_directory() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    format!("➜ {} ", current_directory).chars().count()
}

pub fn redirect_to_home() -> std::io::Result<String> {
    let home = env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    env::set_current_dir(&home)?;

    let dir_name = home
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("/")
        .to_string();

    Ok(dir_name)
}

enum Colors {
    YELLOW(String),
}

impl Colors {
    fn to_ansi(&self) -> String {
        match self {
            Colors::YELLOW(text) => format!("\x1b[1;31m{}\x1b[0m", text),
        }
    }
}

pub fn get_current_directory() -> Result<String> {
    match std::env::current_dir() {
        Ok(path) => match path.file_name() {
            Some(name) => Ok(name.to_string_lossy().to_string()),
            None => Ok("/".to_string()),
        },
        Err(_) => match redirect_to_home() {
            Ok(val) => Ok(val),
            Err(e) => Err(e),
        },
    }
}
