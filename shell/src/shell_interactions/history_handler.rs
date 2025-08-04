use crate::OutputTarget;
use crate::events_handler::{CursorPosition, Shell};
use crate::features::history::History;
use crate::shell_interactions::utils::clear_buff_ter;
use crate::shell_interactions::utils::{calc_termlines_in_buffer, clear_current_line, print_out};
use crate::{display_promt, shell1::*};
use std::io::*;
use termion::cursor::{Down, Up};
use termion::raw::RawTerminal;

impl Shell {
    pub fn load_history_prev(&mut self) {
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
        };
        // self.buffer
        let prev_history = self.history.prev();
        if prev_history != self.buffer {
            clear_buff_ter(stdout, self.buffer.clone());
        }
        if !prev_history.is_empty() {
            self.buffer = prev_history;
            self.cursor_position.reset();
            self.rerender();
        }
    }

    pub fn load_history_next(&mut self) {
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
        };
        let next_history = self.history.next();
        if next_history != self.buffer {
            clear_buff_ter(stdout, self.buffer.clone());
        }
        self.buffer = next_history; // Empty string if no next history
        self.cursor_position.reset();
        self.rerender();
    }
}
