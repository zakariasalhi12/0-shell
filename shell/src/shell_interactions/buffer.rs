use crate::OutputTarget;
use crate::events_handler::Shell;
use crate::shell_interactions::utils::*;
use std::io::*;
use termion::raw::RawTerminal;
use termion::{clear, cursor};
use unicode_width::UnicodeWidthStr;

impl Shell {

    pub fn delete_char(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        if self.cursor_position.x == 0 {
            // Delete at end of buffer (backspace from end)
            self.buffer.pop();
            self.rerender();
        } else {
            // Delete character before cursor position
            let delete_pos_chars = self.buffer.chars().count() - self.cursor_position.x as usize;
            if delete_pos_chars > 0 {
                // Find byte offset of the char we want to delete
                if let Some((byte_index, _)) = self.buffer.char_indices().nth(delete_pos_chars - 1) {
                    self.buffer.remove(byte_index);
                }
            }
            self.rerender();
        }
    }
    pub fn print_out_static(stdout: &mut Option<RawTerminal<Stdout>>, input: &str) {
        match stdout {
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

    pub fn ctrl(&mut self) {
    let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
            _ => {
                return;
            }
        };

        self.buffer.clear();
        Self::print_out_static(stdout , "^C \n\r");
        display_promt(stdout);
    }

    pub fn insert_char(&mut self, c: char) {
        let insert_pos_chars = self.buffer.chars().count() - self.cursor_position.x as usize;
        let insert_pos_bytes = self
            .buffer
            .char_indices()
            .nth(insert_pos_chars)
            .map(|(i, _)| i)
            .unwrap_or(self.buffer.len());

        self.buffer.insert(insert_pos_bytes, c);
        self.rerender();
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position.x < UnicodeWidthStr::width(self.buffer.as_str()) as u16 {
            self.cursor_position.x += 1;
            self.rerender();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position.x > 0 {
            self.cursor_position.x -= 1;
            self.rerender();
        }
    }
    pub fn clear_screen(&mut self) {
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
            _ => {
                return;
            }
        };
        self.buffer.clear();
        self.cursor_position.reset();
        Self::print_out_static(stdout, &format!("{}{}", clear::All, cursor::Goto(1, 1)));
        display_promt(stdout);
    }
}
