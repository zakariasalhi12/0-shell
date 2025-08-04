// use termion::cursor::{Down, Left};
// use termion::raw::RawTerminal;
// use std::io::*;
// use crate::shell_interactions::utils::{calc_termlines_in_buffer, edit_buffer, print_out};
// use crate::{shell::*};

// pub fn push_to_buffer(stdout: &mut Option<RawTerminal<Stdout>>, c: char, buffer: &mut String) {
//     buffer.push(c); // push the character to the buffer
//     print_out(stdout, &format!("{}", c)); // write the character to stdout
// }

// pub fn push(
//     stdout: &mut Option<RawTerminal<Stdout>>,
//     buffer: &mut String,
//     cursor_position: &mut CursorPosition,
//     (x, y): (u16, u16),
//     (width, height): (u16, u16),
//     c: char,
// ) {
//     if cursor_position.x > 0 {
//         if x == width {
//             cursor_position.y -= 1;
//             print_out(stdout, &format!("{}{}", Down(1), Left(width)));
//         }

//         edit_buffer(stdout, c, buffer, cursor_position);
//     } else {
//         let old_lines_in_buffer = calc_termlines_in_buffer(buffer.len());
//         let new_lines_in_buffer = calc_termlines_in_buffer(buffer.len() + 1);

//         if new_lines_in_buffer > old_lines_in_buffer && cursor_position.y != 0 {
//             cursor_position.y -= 1;
//         }
//         push_to_buffer(stdout, c, buffer);
//     }
// }
