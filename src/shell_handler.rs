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

    pub fn push_to_buffer(stdout: &mut RawTerminal<Stdout>, c: char, buffer: &mut String) {
        buffer.push(c); // push the character to the buffer
        write!(stdout, "{}", c).unwrap(); // write the character to stdout
    }

    pub fn pop_from_buffer(stdout: &mut RawTerminal<Stdout>, buffer: &mut String , size : usize) {
        for _ in 0..size {
            if !buffer.is_empty() {
                buffer.pop();
                write!(stdout, "\x08 \x08").unwrap(); // backspace
            }
        }
    }

    pub fn clear_terminal(stdout: &mut RawTerminal<Stdout>) {
        write!(stdout, "{}{}\r", clear::All, cursor::Goto(1, 1)).unwrap();
        display_promt(stdout);
        stdout.flush().unwrap();
    }

    pub fn parse_and_exec(
        stdout: &mut RawTerminal<Stdout>,
        buffer: &mut String,
        history: &mut History,
    ) {
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

    pub fn history_prev(
        stdout: &mut RawTerminal<Stdout>,
        buffer: &mut String,
        history: &mut History,
    ) {
        let prev_history = history.prev();
        if !prev_history.is_empty() {
            Shell::pop_from_buffer(stdout, buffer,buffer.len());
            write!(stdout, "\r").unwrap();
            stdout.flush().unwrap();
            display_promt(stdout);
            write!(stdout, "{}", prev_history).unwrap();
            buffer.push_str(&prev_history);
        }
    }

    pub fn history_next(
        stdout: &mut RawTerminal<Stdout>,
        buffer: &mut String,
        history: &mut History,
    ) {
        let next_history = history.next();
        if !next_history.is_empty() {
            Shell::pop_from_buffer(stdout, buffer , buffer.len());
            buffer.clear();
            write!(stdout, "\r").unwrap();
            display_promt(stdout);
            write!(stdout, "{}", next_history).unwrap();
            buffer.push_str(&next_history);
        }
    }

    pub fn run(&mut self) {
        display_promt(&mut self.stdout);
        self.stdout.flush().unwrap();

        let stdin = &self.stdin;

        for key in stdin.keys() {
            match key.unwrap() {
                termion::event::Key::Char('\n') => {
                    Shell::parse_and_exec(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Char('\t') => {
                    //
                }

                termion::event::Key::Char(c) => {
                    Shell::push_to_buffer(&mut self.stdout, c, &mut self.buffer);
                }

                termion::event::Key::Backspace => {
                    Shell::pop_from_buffer(&mut self.stdout, &mut self.buffer , 1);
                }

                termion::event::Key::Up => {
                    Shell::history_prev(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Down => {
                    Shell::history_next(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Ctrl('l') => {
                    Shell::clear_terminal(&mut self.stdout);
                }

                termion::event::Key::Ctrl('d') => {
                    write!(self.stdout, "\r").unwrap();
                    self.stdout.flush().unwrap();
                    return;
                }

                // termion::event::Key::Ctrl('c') => {
                //     write!(self.stdout, "\r").unwrap();
                //     break;
                // }
                _ => {}
            }
            self.stdout.flush().unwrap();
        }
    }
}
