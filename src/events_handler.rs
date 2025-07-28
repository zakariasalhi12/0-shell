use crate::features::history;
use crate::features::history::History;
use crate::{display_promt, promt_len};
use crate::{executer, parse};
use std::io::*;
use std::{self};
use termion::cursor::{DetectCursorPos, Left, Up};
use termion::cursor::{Down, Goto, Right};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::{clear, cursor};
use whoami::Width;

#[derive(Debug, Clone, Copy)]
pub struct CursorPostition {
    x: u16,
    y: u16,
}

impl CursorPostition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x: x, y: y }
    }
}

pub struct Shell {
    pub stdout: Option<RawTerminal<Stdout>>,
    pub stdin: Stdin,
    pub buffer: String,
    pub history: History,
    pub cursor_position: CursorPostition,
    pub buffer_lines: u16, // How many lines the buffer has in the terminal
    pub need_to_up: bool,  // If the cursor need to go up
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
            cursor_position: CursorPostition::new(0, 0),
            buffer_lines: 0,
            need_to_up: false,
        })
    }

    pub fn re_render(
        stdout: &mut Option<RawTerminal<Stdout>>,
        old_buffer: &mut String,
        new_buffer: String,
        cursor_position: CursorPostition,
    ) {
        if old_buffer.is_empty() || new_buffer.is_empty() {
            return;
        }
        let (x, y) = stdout.as_mut().unwrap().cursor_pos().unwrap(); // get current cursor position
        let (_, height) = termion::terminal_size().unwrap(); // get terminal size
        let free_lines =  (height - y) - cursor_position.y;

        print_out(stdout, &format!("{}", Goto (1, height - free_lines))); // move the cursor to the last line

        for i in 0..calc_termlines_in_buffer(old_buffer.len()) {
            // clear all buffer lines
            if i > 0 {
                print_out(stdout, &format!("{}", Up(1)));
            }
            clear_current_line(stdout);
        }
        old_buffer.clear();
        display_promt(stdout);
        print_out(stdout, &format!("{}{}", new_buffer, Goto(x, y))); // restore the old cursor position
    }

    pub fn push_to_buffer(stdout: &mut Option<RawTerminal<Stdout>>, c: char, buffer: &mut String) {
        buffer.push(c); // push the character to the buffer
        print_out(stdout, &format!("{}", c)); // write the character to stdout
    }

    pub fn pop_from_buffer(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        size: usize,
    ) {
        for _ in 0..size {
            if !buffer.is_empty() {
                buffer.pop();
                print_out(stdout, "\x08 \x08"); // backspace
            }
        }
    }

    pub fn move_cursor_left(
        stdout: &mut Option<RawTerminal<Stdout>>,
        cursor_position: &mut CursorPostition,
        buffer: String,
        buffer_lines: &mut u16,
        need_to_up: &mut bool,
    ) {
        *buffer_lines = calc_termlines_in_buffer(buffer.len());

        let (x, _) = stdout.as_mut().unwrap().cursor_pos().unwrap_or((1, 1));

        if x != 1 && cursor_position.x < buffer.len() as u16 {
            cursor_position.x += 1;
            print_out(stdout, &format!("{}", Left(1)));
        }

        if x == 1 && *need_to_up && *buffer_lines > cursor_position.y {
            let (width, _) = termion::terminal_size().unwrap_or((80, 24));
            cursor_position.y += 1;
            cursor_position.x += 1;
            print_out(stdout, &format!("{}{}", Up(1), Right(width)));
            *need_to_up = false;
        }

        if x == 1 && !*need_to_up && calc_termlines_in_buffer(buffer.len()) > 1 {
            *need_to_up = true;
        }
    }

    pub fn move_cursor_right(
        stdout: &mut Option<RawTerminal<Stdout>>,
        cursor_position: &mut CursorPostition,
        buffer: String,
        buffer_lines: &mut u16,
        need_to_up: &mut bool,
    ) {
        if cursor_position.x > 0 {
            cursor_position.x -= 1;
            print_out(stdout, &format!("{}", Right(1)));
        }

        let (x, _) = stdout.as_mut().unwrap().cursor_pos().unwrap_or((1, 1));
        let (width, _) = termion::terminal_size().unwrap_or((80, 24));

        if x == width && *buffer_lines > 1 && cursor_position.y != 0 {
            print_out(stdout, &format!("{}{}", Down(1), Left(width)));
            cursor_position.y -= 1;
            cursor_position.x -= 1;
        }
    }

    // if the character == \0 remove the character from the buffer instead of add it
    pub fn edit_buffer(
        stdout: &mut Option<RawTerminal<Stdout>>,
        character: char,
        buffer: &mut String,
        cursor_position: CursorPostition,
    ) {
        let mut remove: i16 = 0;
        if character == '\0' {
            remove = -1
        }

        let mut res = String::new();
        for (i, c) in buffer.to_owned().char_indices() {
            if (i as i16) == (buffer.len() as i16) - cursor_position.x as i16 + remove {
                if character == '\0' {
                    continue;
                }
                res.push(character);
            }
            res.push(c);
        }
        Shell::re_render(stdout, buffer, res.clone(), cursor_position);
        buffer.clear();
        if remove == -1 {
            print_out(stdout, &format!("{}", Left(1)));
        } else {
            print_out(stdout, &format!("{}", Right(1)));
        }
        buffer.push_str(&res);
    }

    pub fn history_prev(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        history: &mut History,
    ) {
        let prev_history = history.prev();
        if !prev_history.is_empty() {
            for i in 0..calc_termlines_in_buffer(buffer.len()) {
                if i > 0 {
                    print_out(stdout, &format!("{}", Up(1)));
                }
                clear_current_line(stdout);
            }
            buffer.clear();
            display_promt(stdout);
            print_out(stdout, &prev_history);
            buffer.push_str(&prev_history);
        }
    }

    pub fn history_next(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        history: &mut History,
    ) {
        let next_history = history.next();
        if !next_history.is_empty() {
            for i in 0..calc_termlines_in_buffer(buffer.len()) {
                if i > 0 {
                    print_out(stdout, &format!("{}", Up(1)));
                }
                clear_current_line(stdout);
            }
            buffer.clear();
            display_promt(stdout);
            print_out(stdout, &next_history);
            buffer.push_str(&next_history);
        }
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
        display_promt(stdout);
    }

    pub fn run(&mut self) {
        display_promt(&mut self.stdout);

        let stdin = &self.stdin;

        for key in stdin.keys() {
            match key.unwrap() {
                // Parse Input
                termion::event::Key::Char('\n') => {
                    self.cursor_position = CursorPostition::new(0, 0);
                    self.need_to_up = false;
                    Shell::parse_and_exec(&mut self.stdout, &mut self.buffer, &mut self.history);
                }

                termion::event::Key::Char('\t') => {
                    //
                }
                // append character to the buffer and write it in the stdout
                termion::event::Key::Char(c) => {
                    let buffer_len = self.buffer.len();
                    if self.cursor_position.x > 0 {
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
                    let (x, _) = self.stdout.as_mut().unwrap().cursor_pos().unwrap_or((1, 1));
                    let (width, _) = termion::terminal_size().unwrap_or((80, 24));

                    if x == 1 && self.cursor_position.y <= calc_termlines_in_buffer(self.buffer.len()) + 1{
                        self.cursor_position.y += 1;
                        print_out(&mut self.stdout, &format!("{}{}", Up(1), Right(width)));
                    }

                    if self.cursor_position.x > 0 {
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
                    self.need_to_up = false;
                    self.cursor_position = CursorPostition::new(0, 0);
                }

                // Get next history
                termion::event::Key::Down => {
                    Shell::history_next(&mut self.stdout, &mut self.buffer, &mut self.history);
                    self.need_to_up = false;
                    self.cursor_position = CursorPostition::new(0, 0);
                }

                // Move the cursor to the right
                termion::event::Key::Left => {
                    Shell::move_cursor_left(
                        &mut self.stdout,
                        &mut self.cursor_position,
                        self.buffer.clone(),
                        &mut self.buffer_lines,
                        &mut self.need_to_up,
                    );
                }

                // Move the cursor to the left
                termion::event::Key::Right => {
                    Shell::move_cursor_right(
                        &mut self.stdout,
                        &mut self.cursor_position,
                        self.buffer.clone(),
                        &mut self.buffer_lines,
                        &mut self.need_to_up,
                    );
                }
                // Clear terminal
                termion::event::Key::Ctrl('l') => {
                    self.cursor_position = CursorPostition::new(0, 0);
                    self.need_to_up = false;
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

pub fn clear_terminal(stdout: &mut Option<RawTerminal<Stdout>>, buffer: &mut String) {
    buffer.clear();
    print_out(stdout, &format!("{}{}\r", clear::All, cursor::Goto(1, 1)));
    display_promt(stdout);
}

fn clear_current_line(stdout: &mut Option<RawTerminal<Stdout>>) {
    print_out(stdout, &format!("{}\r", clear::CurrentLine));
}

fn calc_termlines_in_buffer(buffer_size: usize) -> u16 {
    let (width, _) = termion::terminal_size().unwrap_or((80, 24));
    (width + ((buffer_size + promt_len()) as u16 - 1)) / width
}

pub fn print_out(w: &mut Option<RawTerminal<Stdout>>, input: &str) {
    match w {
        Some(raw_stdout) => {
            write!(raw_stdout, "{}", input).unwrap();
            raw_stdout.flush().unwrap();
        }
        None => {
            let mut std = std::io::stdout();
            write!(std, "{}", input).unwrap();
            std.flush().unwrap();
        }
    }
}
