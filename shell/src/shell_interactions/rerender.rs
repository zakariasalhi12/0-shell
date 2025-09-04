use crate::OutputTarget;
use crate::events_handler::Shell;
use crate::shell_interactions::utils::prompt_len;
use crate::shell_interactions::utils::*;
use std::{self};
use termion::cursor::Goto;
use termion::raw::RawTerminal;
use termion::{clear, cursor};
use unicode_width::UnicodeWidthStr;

impl Shell {
    pub fn rerender(&mut self) {
        let (term_width, _term_height) = termion::terminal_size().unwrap_or((80, 24));
        let prompt_length = prompt_len() as u16;
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
            _ => {
                return;
            }
        };

        // Store values we need to avoid borrowing conflicts
        let buffer_clone = self.buffer.clone();
        let buffer_len = UnicodeWidthStr::width(self.buffer.as_str());
        let cursor_x = self.cursor_position.x;

        // Calculate how many lines the current buffer will occupy
        let total_content_len = prompt_length + buffer_len as u16;
        let total_lines = if total_content_len == 0 {
            1
        } else {
            (total_content_len + term_width - 1) / term_width
        };

        // Clear the current line and any additional lines that might contain our content
        Self::print_out_static(stdout, &format!("\r{}", clear::CurrentLine));

        // Clear additional lines if content spans multiple lines
        if total_lines > 1 {
            for _ in 1..total_lines {
                Self::print_out_static(
                    stdout,
                    &format!("{}{}", cursor::Down(1), clear::CurrentLine),
                );
            }
            // Move back to the original line
            for _ in 1..total_lines {
                Self::print_out_static(stdout, &format!("{}", cursor::Up(1)));
            }
        }

        // Move to beginning of line and redraw prompt and buffer
        Self::print_out_static(stdout, &format!("\r"));
        display_promt(stdout);

        if !buffer_clone.is_empty() {
            Self::print_out_static(stdout, &buffer_clone);
        }

        // Calculate cursor position in buffer
        let cursor_position_in_buffer = if cursor_x > buffer_len as u16 {
            0
        } else {
            buffer_len - cursor_x as usize
        };

        // Total position in the terminal line (including prompt)
        let total_cursor_pos = prompt_length as usize + cursor_position_in_buffer;

        // Calculate which line and column the cursor should be on
        let cursor_line_offset = total_cursor_pos / term_width as usize;
        let cursor_col = (total_cursor_pos % term_width as usize) + 1;

        // Update cursor position tracking
        self.cursor_position.y = cursor_line_offset as u16;

        // Position cursor correctly
        if cursor_line_offset > 0 {
            // Move cursor to the correct line
            Self::print_out_static(
                stdout,
                &format!("{}", cursor::Down(cursor_line_offset as u16)),
            );
        }
        Self::print_out_static(
            stdout,
            &format!("\r{}", cursor::Right(cursor_col as u16 - 1)),
        );
    }
}
