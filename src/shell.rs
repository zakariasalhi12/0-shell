use crate::features::history;
use crate::features::history::History;
use crate::shell_interactions::edit_buffer::push;
use crate::shell_interactions::history_handler::{history_next, history_prev};
use crate::shell_interactions::pop_from_buffer::pop;
use crate::shell_interactions::positions_handler::{move_cursor_left, move_cursor_right};
use crate::shell_interactions::utils::{clear_terminal, print_out};
use crate::{display_prompt, prompt_len, shell};
use crate::{executer, parse};
use std::io::*;
use std::{self};
use termion::cursor::{DetectCursorPos, Left, Up};
use termion::cursor::{Down, Goto, Right};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::{clear, cursor};

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

impl CursorPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x: x, y: y }
    }
}

pub struct Shell {
    pub stdout: Option<RawTerminal<Stdout>>,
    pub stdin: Stdin,
    pub buffer: String,
    pub history: History,
    pub cursor_position: CursorPosition,
}

impl Shell {
    pub fn new() -> std::io::Result<Self> {
        let stdout = match stdout().into_raw_mode() {
            Ok(raw) => Some(raw),
            Err(_) => {
                eprintln!("stdout is not a TTY (maybe piped?). Raw mode not available.");
                None
            }
        };

        Ok(Shell {
            stdin: stdin(),
            stdout,
            buffer: String::new(),
            history: history::History::new(),
            cursor_position: CursorPosition::new(0, 0),
        })
    }

    pub fn parse_and_exec(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        history: &mut History,
    ) {
        match stdout {
            Some(s) => {
                writeln!(s).unwrap();
                s.flush().unwrap();
            }
            None => {
                writeln!(std::io::stdout()).unwrap();
                std::io::stdout().flush().unwrap();
            }
        }

        print!("\r\x1b[2K");
        std::io::stdout().flush().unwrap();

        if !buffer.trim().is_empty() {
            history.save(buffer.clone());
            let cmd = parse(&buffer);
            executer::execute(cmd);
        }

        buffer.clear();
        display_prompt(stdout);
    }

    pub fn run(&mut self) {
        display_prompt(&mut self.stdout);

        let stdin = &self.stdin;
        let mut need_to_up = false;
        let mut need_to_down = false;
        let mut buffer_lines = 0;

        for key in stdin.keys() {
            let (x, y) = self.stdout.as_mut().unwrap().cursor_pos().unwrap_or((1, 1));
            let (width, height) = termion::terminal_size().unwrap_or((80, 24));

            match key.unwrap() {
                // Parse Input
                termion::event::Key::Char('\n') => {
                    self.cursor_position = CursorPosition::new(0, 0);
                    need_to_up = false;
                    Shell::parse_and_exec(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Char('\t') => {
                    //
                }
                // append character to the buffer and write it in the stdout
                termion::event::Key::Char(c) => {
                    push(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.cursor_position,
                        (x, y),
                        (width, height),
                        c,
                    );
                }

                // Remove the last character
                termion::event::Key::Backspace => {
                    pop(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.cursor_position,
                        (x, y),
                        (width, height),
                    );
                }

                // Get prev history
                termion::event::Key::Up => {
                    history_prev(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.history,
                        self.cursor_position,
                    );
                    need_to_up = false;
                    self.cursor_position = CursorPosition::new(0, 0);
                }

                // Get next history
                termion::event::Key::Down => {
                    history_next(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.history,
                        self.cursor_position,
                    );
                    need_to_up = false;
                    self.cursor_position = CursorPosition::new(0, 0);
                }

                // Move the cursor to the right
                termion::event::Key::Left => {
                    move_cursor_left(
                        &mut self.stdout,
                        &mut self.cursor_position,
                        self.buffer.clone(),
                        &mut buffer_lines,
                        &mut need_to_up,
                        (x, y),
                        (width, height),
                    );
                }

                // Move the cursor to the left
                termion::event::Key::Right => {
                    move_cursor_right(
                        &mut self.stdout,
                        &mut self.cursor_position,
                        self.buffer.clone(),
                        &mut buffer_lines,
                        &mut need_to_down,
                        (x, y),
                        (width, height),
                    );
                }
                // Clear terminal
                termion::event::Key::Ctrl('l') => {
                    self.cursor_position = CursorPosition::new(0, 0);
                    need_to_up = false;
                    clear_terminal(&mut self.stdout, &mut self.buffer);
                }

                // Kill terminal proc
                termion::event::Key::Ctrl('d') => {
                    print_out(&mut self.stdout, "\r");
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
        }
    }
}

// Some utility functions

