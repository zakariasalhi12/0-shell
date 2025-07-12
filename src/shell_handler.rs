use std::io::*;
use termion::cursor::Goto;
use termion::{clear , cursor};

use shell::display_promt;
use shell::features::history;
use shell::features::history::History;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

use crate::executer::execute;
use crate::{executer, parse};

pub struct Shell {
    pub stdout: RawTerminal<Stdout>,
    pub stdin: Stdin,
    pub buffer: String,
    pub history: History,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            buffer: String::new(),
            history: history::History::new(),
        }
    }

    pub fn push_to_buffer(&mut self, c: char) {
        self.buffer.push(c); // push the character to the buffer
        write!(self.stdout, "{}", c).unwrap(); // write the character to stdout
        self.stdout.flush().unwrap(); // transfer data from the buffer to the stdout
    }

    pub fn pop_from_buffer(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            write!(self.stdout, "\x08 \x08").unwrap(); // backspace
            self.stdout.flush().unwrap();
        }
    }

    pub fn run(&mut self) {
        display_promt(&mut self.stdout);
        self.stdout.flush().unwrap();
        let stdin = &self.stdin;
        for key in stdin.keys() {
            match key.unwrap() {
                termion::event::Key::Char('\n') => {
                    writeln!(self.stdout).unwrap();
                    print!("\r\x1b[2K");
                    if !self.buffer.trim().is_empty() {
                        self.history.save(self.buffer.clone());
                        let cmd = parse(&self.buffer);
                        executer::execute(cmd);
                    }
                    self.buffer.clear();
                    display_promt(&mut self.stdout);
                }

                termion::event::Key::Char(c) => {
                    self.buffer.push(c); // push the character to the buffer
                    write!(self.stdout, "{}", c).unwrap(); // write the character to stdout
                }

                termion::event::Key::Backspace => {
                    if !self.buffer.is_empty() {
                        self.buffer.pop();
                        write!(self.stdout, "\x08 \x08").unwrap(); // backspace
                    }
                }

                termion::event::Key::Ctrl('c') => {
                    write!(self.stdout, "\r").unwrap();
                    break;
                }

                termion::event::Key::Up => {
                    let next_history = self.history.next();
                    if !next_history.is_empty() {
                        self.buffer.clear();
                        write!(self.stdout , "\r").unwrap();
                        // ANSI escape code to clear the current line and move the cursor to the beginning
                        self.stdout.flush().unwrap();
                        display_promt(&mut self.stdout);
                        write!(self.stdout, "{}", next_history).unwrap();
                    }
                }
                
                termion::event::Key::Down => {
                    let prev_history = self.history.prev();
                    if !prev_history.is_empty() {
                        self.buffer.clear();
                        write!(self.stdout , "\r").unwrap();
                        // ANSI escape code to clear the current line and move the cursor to the beginning
                        display_promt(&mut self.stdout);
                        write!(self.stdout, "{}", prev_history).unwrap();
                    }
                }

                termion::event::Key::Ctrl('l') => {
                    write!(self.stdout, "{}{}\r", clear::All , cursor::Goto(1 , 1)).unwrap();
                    display_promt(&mut self.stdout);
                    self.stdout.flush().unwrap();
                }

                termion::event::Key::Ctrl('d') => {
                    write!(self.stdout , "\r").unwrap();
                    self.stdout.flush().unwrap();
                    return;
                }   
                _ => {}
                
            }
            self.stdout.flush().unwrap();
        }
    }
}
