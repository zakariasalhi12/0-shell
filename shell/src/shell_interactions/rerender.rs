use crate::shell_interactions::utils::prompt_len;
use crate::OutputTarget;
use crate::events_handler::Shell;
use std::{self};
use termion::cursor::DetectCursorPos;
use termion::cursor::Goto;
use termion::raw::RawTerminal;
use termion::{clear, cursor};
use unicode_width::UnicodeWidthStr;
use crate::shell_interactions::utils::*;

impl Shell {
    pub fn rerender(&mut self) {
        let (term_width, _term_height) = termion::terminal_size().unwrap_or((80, 24));
        let prompt_length = prompt_len() as u16;
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
             _=>{
                return;
            }
        };
        // Store values we need to avoid borrowing conflicts
        let buffer_clone = self.buffer.clone();
        let buffer_len = UnicodeWidthStr::width(self.buffer.as_str());
        let cursor_x = self.cursor_position.x;
        let cursor_y = self.cursor_position.y;

        // Calculate how many lines the current buffer will occupy
        let total_content_len = prompt_length + buffer_len as u16;
        let new_total_lines = if total_content_len == 0 {
            1
        } else {
            (total_content_len + term_width - 1) / term_width
        };

        let (_current_x, current_y) = if let Some(std) = stdout {
            let (x, y) = match std.cursor_pos() {
                Ok(pos) => pos,
                Err(_) => return, // 
            };
            (x,y)
        } else {
            return;
        };
        // Calculate where the prompt starts
        let prompt_start_line = current_y.saturating_sub(cursor_y);

        Self::print_out_static(stdout, &format!("{}", Goto(1, prompt_start_line)));

        // Calculate how many lines we might need to clear (be generous)
        let lines_to_clear = (cursor_y + 3).max(new_total_lines + 2).max(5);

        // Clear from prompt line downward
        for i in 0..lines_to_clear {
            Self::print_out_static(stdout, &format!("{}", clear::CurrentLine));
            if i < lines_to_clear - 1 {
                Self::print_out_static(stdout, &format!("{}", cursor::Down(1)));
            }
        }

        // Go back to prompt line and redraw everything
        Self::print_out_static(stdout, &format!("{}", Goto(1, prompt_start_line)));
        display_promt(stdout);

        if !buffer_clone.is_empty() {
            Self::print_out_static(stdout, &buffer_clone);
        }

        let cursor_position_in_buffer = if cursor_x > buffer_len as u16 {
            0 // Clamp to start if somehow cursor_x is too large
        } else {
            buffer_len - cursor_x as usize
        };

        // Total position in the terminal line (including prompt)
        let total_cursor_pos = prompt_length as usize + cursor_position_in_buffer;

        // Calculate which line and column the cursor should be on
        let cursor_line_offset = total_cursor_pos / term_width as usize;
        let cursor_col = (total_cursor_pos % term_width as usize) + 1;

        let final_line = prompt_start_line + cursor_line_offset as u16;

        // Update cursor position tracking
        self.cursor_position.y = cursor_line_offset as u16;

        // Move cursor to final position
        Self::print_out_static(stdout, &format!("{}", Goto(cursor_col as u16, final_line)));
    }
}
