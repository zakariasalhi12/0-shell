use crate::{executer, parse};
use shell::display_promt;
use shell::features::history;
use shell::features::history::History;
use std::io::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::{clear, cursor};

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

    // pub fn push_to_buffer(stdout : &mut RawTerminal<Stdout>, c: char , buffer : &mut String ) {
    //     buffer.push(c); // push the character to the buffer
    //     write!(self.stdout, "{}", c).unwrap(); // write the character to stdout
    // }

    pub fn pop_from_buffer(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            write!(self.stdout, "\x08 \x08").unwrap(); // backspace
            self.stdout.flush().unwrap();
        }
    }

    pub fn parse_and_exec(stdout : &mut RawTerminal<Stdout> , buffer : &mut String , history : &mut History) {
        writeln!(stdout).unwrap();
        print!("\r\x1b[2K");
        if !buffer.trim().is_empty() {
            history.save(buffer.clone());
            let cmd = parse(&buffer);
            executer::execute(cmd);
        }
        buffer.clear();
        display_promt(stdout);
    }

    pub fn run(&mut self) {
        display_promt(&mut self.stdout);
        self.stdout.flush().unwrap();

        let stdin = &self.stdin;

        for key in stdin.keys() {
            match key.unwrap() {

                termion::event::Key::Char('\n') => {
                    Shell::parse_and_exec(&mut self.stdout , &mut self.buffer , &mut self.history);
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
                    // write!(self.stdout, "\r").unwrap();
                    // break;
                }

                termion::event::Key::Up => {
                    let next_history = self.history.next();
                    if !next_history.is_empty() {
                        self.buffer.clear();
                        write!(self.stdout, "\r").unwrap();
                        self.stdout.flush().unwrap();
                        display_promt(&mut self.stdout);
                        write!(self.stdout, "{}", next_history).unwrap();
                        self.buffer.push_str(&next_history);
                    }
                }

                termion::event::Key::Down => {
                    let prev_history = self.history.prev();
                    if !prev_history.is_empty() {
                        self.buffer.clear();
                        write!(self.stdout, "\r").unwrap();
                        display_promt(&mut self.stdout);
                        write!(self.stdout, "{}", prev_history).unwrap();
                        self.buffer.push_str(&prev_history);
                    }
                }

                termion::event::Key::Ctrl('l') => {
                    write!(self.stdout, "{}{}\r", clear::All, cursor::Goto(1, 1)).unwrap();
                    display_promt(&mut self.stdout);
                    self.stdout.flush().unwrap();
                }

                termion::event::Key::Ctrl('d') => {
                    write!(self.stdout, "\r").unwrap();
                    self.stdout.flush().unwrap();
                    return;
                }
                _ => {}
            }
            self.stdout.flush().unwrap();
        }
    }
}
