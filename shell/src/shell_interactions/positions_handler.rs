use crate::shell_interactions::utils::{calc_termlines_in_buffer, print_out};
use crate::shell1::*;
// use events_handler::*;
use crate::events_handler::CursorPosition;
use std::io::*;
use termion::cursor::{Down, Left, Right, Up};
use termion::raw::RawTerminal;

pub fn move_cursor_left(
    stdout: &mut Option<RawTerminal<Stdout>>,
    cursor_position: &mut CursorPosition,
    buffer: String,
    buffer_lines: &mut u16,
    need_to_up: &mut bool,
    (x, y): (u16, u16),
    (width, _): (u16, u16),
) {
    *buffer_lines = calc_termlines_in_buffer(buffer.len());

    if x != 1 && cursor_position.x < buffer.len() as u16 {
        cursor_position.x += 1;
        print_out(stdout, &format!("{}", Left(1)));
    }

    if x == 1 && !*need_to_up && calc_termlines_in_buffer(buffer.len()) > 1 {
        *need_to_up = true;
    }

    if x == 1 && *need_to_up && *buffer_lines > cursor_position.y {
        cursor_position.y += 1;
        cursor_position.x += 1;
        print_out(stdout, &format!("{}{}", Up(1), Right(width)));
        *need_to_up = false;
    }
}

pub fn move_cursor_right(
    stdout: &mut Option<RawTerminal<Stdout>>,
    cursor_position: &mut CursorPosition,
    buffer: String,
    buffer_lines: &mut u16,
    need_to_down: &mut bool,
    (x, y): (u16, u16),
    (width, _): (u16, u16),
) {
    // leb make
    if cursor_position.x > 0 {
        cursor_position.x = if let Some(val) = cursor_position.x.checked_sub(1) {
            val
        } else {
            cursor_position.x
        };
        print_out(stdout, &format!("{}", Right(1)));
    }

    if x == width && !*need_to_down && cursor_position.y < *buffer_lines {
        *need_to_down = true;
    } else if *need_to_down && x == width && *buffer_lines > 1 && cursor_position.y != 0 {
        print_out(stdout, &format!("{}{}", Down(1), Left(width)));
        cursor_position.y = if let Some(val) = cursor_position.y.checked_sub(1) {
            val
        } else {
            cursor_position.y
        };
        cursor_position.x = if let Some(val) = cursor_position.x.checked_sub(1) {
            val
        } else {
            cursor_position.x
        };
        *need_to_down = false;
    }
}
