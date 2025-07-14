use crate::display_promt;
use crate::features::history;
use crate::features::history::History;
use crate::{executer, parse};
use std::io::*;
use termion::cursor::{Left};
use termion::cursor::Right;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::{clear, cursor};

pub struct Shell {
    pub stdout: RawTerminal<Stdout>,
    pub stdin: Stdin,
    pub buffer: String,
    pub history: History,
    pub cursor_position: i16,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            buffer: String::new(),
            history: history::History::new(),
            cursor_position: 0,
        }
    }

    pub fn push_to_buffer(stdout: &mut RawTerminal<Stdout>, c: char, buffer: &mut String) {
        buffer.push(c); // push the character to the buffer
        write!(stdout, "{}", c).unwrap(); // write the character to stdout
    }

    pub fn pop_from_buffer(stdout: &mut RawTerminal<Stdout>, buffer: &mut String, size: usize) {
        for _ in 0..size {
            if !buffer.is_empty() {
                buffer.pop();
                write!(stdout, "\x08 \x08").unwrap(); // backspace
            }
        }
    }

    // if the character == \0 remove the character from the buffer instead of add it
    pub fn edit_buffer(
        stdout: &mut RawTerminal<Stdout>,
        character: char,
        buffer: &mut String,
        cursor_position: i16,
    ) {
        let mut remove: i16 = 0;
        
        if character == '\0' {
            remove = -1
        }

        let mut res = String::new();
        for (i, c) in buffer.to_owned().char_indices() {
            if (i as i16) == (buffer.len() as i16) - cursor_position + remove {
                if character == '\0' {
                    continue;
                }
                res.push(character);
            }
            res.push(c);
        }
        write!(stdout, "{}", Right(cursor_position as u16)).unwrap();
        Shell::pop_from_buffer(stdout, buffer, buffer.len());
        buffer.push_str(&res);
        write!(stdout, "{}{}", buffer, Left(cursor_position as u16)).unwrap();
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
            Shell::pop_from_buffer(stdout, buffer, buffer.len());
            write!(stdout, "\r").unwrap();
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
            Shell::pop_from_buffer(stdout, buffer, buffer.len());
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
                // Parse Input
                termion::event::Key::Char('\n') => {
                    self.cursor_position = 0;
                    Shell::parse_and_exec(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Char('\t') => {
                    // 
                }

                // append character to the buffer and write it in the stdout
                termion::event::Key::Char(c) => {
                    if self.cursor_position > 0 {
                        Shell::edit_buffer(
                            &mut self.stdout,
                            c,
                            &mut self.buffer,
                            self.cursor_position,
                        );
                    } else {
                        Shell::push_to_buffer(&mut self.stdout, c, &mut self.buffer);
                    }
                }

                // Remove the last character
                termion::event::Key::Backspace => {
                    if self.cursor_position > 0 {
                        Shell::edit_buffer(
                            &mut self.stdout,
                            '\0',
                            &mut self.buffer,
                            self.cursor_position,
                        );
                    } else {
                        Shell::pop_from_buffer(&mut self.stdout, &mut self.buffer, 1);
                    }
                }

                // Get prev history
                termion::event::Key::Up => {
                    Shell::history_prev(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                // Get next history
                termion::event::Key::Down => {
                    Shell::history_next(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                // Move the cursor to the right
                termion::event::Key::Left => {
                    if self.cursor_position < self.buffer.len() as i16 {
                        self.cursor_position += 1;
                        write!(self.stdout, "{}", Left(1)).unwrap();
                    }
                }

                // Move the cursor to the left
                termion::event::Key::Right => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        write!(self.stdout, "{}", Right(1)).unwrap();
                    }
                }

                // Clear terminal
                termion::event::Key::Ctrl('l') => {
                    self.cursor_position += 1;
                    Shell::clear_terminal(&mut self.stdout);
                }

                // Kill terminal proc
                termion::event::Key::Ctrl('d') => {
                    write!(self.stdout, "\r").unwrap();
                    self.stdout.flush().unwrap();
                    return;
                }

                // Remove the whole Word from buffer and delete it from terminal
                termion::event::Key::Ctrl('w') => {
                    //    
                }


                // Send SIGINT signal to the current process (signal number is 2)
                termion::event::Key::Ctrl('c') => {
                    //
                }

                // Send SIGTSTP signal to the current process (signal number is 20)
                termion::event::Key::Ctrl('z') => {
                    //
                }

                _ => {}
            }
            self.stdout.flush().unwrap();
        }
    }
}
