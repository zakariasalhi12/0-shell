use crate::OutputTarget;
use crate::display_promt;
use crate::events_handler::Shell;
use std::io::*;
use termion::raw::RawTerminal;
use termion::{clear, cursor};

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
            let delete_pos = self.buffer.len() - self.cursor_position.x as usize;
            if delete_pos > 0 {
                self.buffer.remove(delete_pos - 1);
                // Don't change cursor_position.x since we deleted before the cursor
            } else {
                return; // Nothing to delete
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

    pub fn insert_char(&mut self, c: char) {
        let insert_pos = self.buffer.len() - self.cursor_position.x as usize;
        self.buffer.insert(insert_pos, c);
        self.rerender();
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position.x < self.buffer.len() as u16 {
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
            _=>{
                return;
            }
        };
        self.buffer.clear();
        self.cursor_position.reset();
        Self::print_out_static(stdout, &format!("{}{}", clear::All, cursor::Goto(1, 1)));
        display_promt(stdout);
    }
}
