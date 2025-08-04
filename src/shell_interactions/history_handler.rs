use termion::cursor::{Down, Up};
use termion::raw::RawTerminal;
use std::io::*;
use crate::shell_interactions::utils::{calc_termlines_in_buffer, clear_current_line, print_out};
use crate::{display_prompt, shell::*};
use crate::features::history::History;

pub fn history_prev(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        history: &mut History,
        cursor_position: CursorPosition,
    ) {
        let prev_history = history.prev();
        if cursor_position.y > 0 {
            for _ in 0..cursor_position.y {
                print_out(stdout, &format!("{}", Down(1)));
            }
        }
        if !prev_history.is_empty() {
            for i in 0..calc_termlines_in_buffer(buffer.len()) {
                if i > 0 {
                    print_out(stdout, &format!("{}", Up(1)));
                }
                clear_current_line(stdout);
            }
            buffer.clear();
            display_prompt(stdout);
            print_out(stdout, &prev_history);
            buffer.push_str(&prev_history);
        }
    }

    pub fn history_next(
        stdout: &mut Option<RawTerminal<Stdout>>,
        buffer: &mut String,
        history: &mut History,
        cursor_position: CursorPosition,
    ) {
        let next_history = history.next();
        if cursor_position.y > 0 {
            for _ in 0..cursor_position.y {
                print_out(stdout, &format!("{}", Down(1)));
            }
        }
        if !next_history.is_empty() {
            for i in 0..calc_termlines_in_buffer(buffer.len()) {
                if i > 0 {
                    print_out(stdout, &format!("{}", Up(1)));
                }
                clear_current_line(stdout);
            }
            buffer.clear();
            display_prompt(stdout);
            print_out(stdout, &next_history);
            buffer.push_str(&next_history);
        }
    }