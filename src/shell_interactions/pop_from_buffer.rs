use termion::{clear, cursor};
use termion::cursor::{ Right, Up};
use termion::raw::RawTerminal;
use std::io::*;
use crate::shell_interactions::utils::{calc_termlines_in_buffer, edit_buffer, print_out};
use crate::{prompt_len, shell::*};


pub fn pop_from_buffer(stdout: &mut Option<RawTerminal<Stdout>>, buffer: &mut String, size: usize) {
    if !buffer.is_empty() {
        for _ in 0..size {
            // if x == width {
            // print_out(stdout, &format!("{}", clear::AfterCursor)); // backspace
            // buffer.pop();
            // } else {
            print_out(
                stdout,
                &format!("{}{}", cursor::Left(1), clear::AfterCursor),
            ); // backspace
            buffer.pop();
            // }
        }
    }
}

pub fn pop(
    stdout: &mut Option<RawTerminal<Stdout>>,
    buffer: &mut String,
    cursor_position: &mut CursorPosition,
    (x, y): (u16, u16),
    (width, height): (u16, u16),
) {
    if buffer.is_empty() {
        return;
    }

    let lines_in_buffer = calc_termlines_in_buffer(buffer.len());
    let lines_in_buffer_after_pop = calc_termlines_in_buffer(buffer.len() - 1);

    if x == 1 && cursor_position.y <= lines_in_buffer - 1 {
        cursor_position.y += 1;
        print_out(stdout, &format!("{}{}", Up(1), Right(width)));
    }

    if x <= prompt_len() as u16 && cursor_position.y == lines_in_buffer - 1 {
        return;
    }

    if cursor_position.x > 0 {
        edit_buffer(stdout, '\0', buffer, cursor_position);
    } else {
        pop_from_buffer(stdout, buffer, 1);
    }

    if lines_in_buffer_after_pop < lines_in_buffer && cursor_position.y != 0 {
        cursor_position.y -= 1;
    }
}
