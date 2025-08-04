// // Enhanced shell.rs with centralized rendering
// use crate::exec::*;
// use crate::features::history;
// use crate::features::history::History;
// use crate::parser;
// use crate::shell_interactions::utils::clear_buff_ter;
// use crate::shell_interactions::utils::{clear_terminal, print_out};
// use crate::{display_promt, prompt_len, shell1};
// use std::io::*;
// use termion::cursor::{DetectCursorPos, Goto, Up};
// use termion::input::TermRead;
// use termion::raw::IntoRawMode;
// use termion::raw::RawTerminal;
// use termion::{clear, cursor};

// #[derive(Debug, Clone, Copy)]
// pub struct CursorPosition {
//     pub x: u16, // Position within the buffer (0 = at end)
//     pub y: u16, // Line offset from prompt line
// }

// impl CursorPosition {
//     pub fn new(x: u16, y: u16) -> Self {
//         Self { x, y }
//     }

//     pub fn reset(&mut self) {
//         self.x = 0;
//         self.y = 0;
//     }
// }

// pub struct Shell {
//     pub stdout: Option<RawTerminal<Stdout>>,
//     pub stdin: Stdin,
//     pub buffer: String,
//     pub history: History,
//     pub cursor_position: CursorPosition,
// }

// impl Shell {
//     pub fn new() -> std::io::Result<Self> {
//         let stdout = match stdout().into_raw_mode() {
//             Ok(raw) => Some(raw),
//             Err(_) => {
//                 eprintln!("stdout is not a TTY (maybe piped?). Raw mode not available.");
//                 None
//             }
//         };

//         Ok(Shell {
//             stdin: stdin(),
//             stdout,
//             buffer: String::new(),
//             history: history::History::new(),
//             cursor_position: CursorPosition::new(0, 0),
//         })
//     }

//     pub fn parse_and_exec(
//         stdout: &mut Option<RawTerminal<Stdout>>,
//         buffer: &mut String,
//         history: &mut History,
//     ) {
//         match stdout {
//             Some(s) => {
//                 writeln!(s).unwrap();
//                 s.flush().unwrap();
//             }
//             None => {
//                 writeln!(std::io::stdout()).unwrap();
//                 std::io::stdout().flush().unwrap();
//             }
//         }

//         print!("\r\x1b[2K");
//         std::io::stdout().flush().unwrap();

//         if !buffer.trim().is_empty() {
//             history.save(buffer.clone());
//             let cmd = parse(&buffer);
//             execute(cmd);
//         }

//         buffer.clear();
//         display_prompt(stdout);
//     }

//     // Centralized rendering function that handles all cases
//     pub fn rerender(&mut self) {
//         let (term_width, _term_height) = termion::terminal_size().unwrap_or((80, 24));
//         let prompt_length = prompt_len() as u16;

//         // Store values we need to avoid borrowing conflicts
//         let buffer_clone = self.buffer.clone();
//         let buffer_len = self.buffer.len();
//         let cursor_x = self.cursor_position.x;
//         let cursor_y = self.cursor_position.y;

//         // Calculate how many lines the current buffer will occupy
//         let total_content_len = prompt_length + buffer_len as u16;
//         let new_total_lines = if total_content_len == 0 {
//             1
//         } else {
//             (total_content_len + term_width - 1) / term_width
//         };

//         // Get current terminal position
//         let (current_x, current_y) = match &mut self.stdout {
//             Some(stdout) => stdout.cursor_pos().unwrap_or((1, 1)),
//             None => (1, 1),
//         };

//         // Calculate where the prompt starts
//         let prompt_start_line = current_y.saturating_sub(cursor_y);

//         // First, we need to clear all lines that might have old content
//         // Go to the start of the prompt line
//         Self::print_out_static(&mut self.stdout, &format!("{}", Goto(1, prompt_start_line)));

//         // Calculate how many lines we might need to clear (be generous)
//         let lines_to_clear = (cursor_y + 3).max(new_total_lines + 2).max(5);

//         // Clear from prompt line downward
//         for i in 0..lines_to_clear {
//             Self::print_out_static(&mut self.stdout, &format!("{}", clear::CurrentLine));
//             if i < lines_to_clear - 1 {
//                 Self::print_out_static(&mut self.stdout, &format!("{}", cursor::Down(1)));
//             }
//         }

//         // Go back to prompt line and redraw everything
//         Self::print_out_static(&mut self.stdout, &format!("{}", Goto(1, prompt_start_line)));
//         display_prompt(&mut self.stdout);

//         if !buffer_clone.is_empty() {
//             Self::print_out_static(&mut self.stdout, &buffer_clone);
//         }

//         // Calculate final cursor position
//         // cursor_x represents how many characters from the END of buffer the cursor is
//         // So if cursor_x = 0, cursor is at the very end
//         // If cursor_x = 2, cursor is 2 characters before the end

//         let cursor_position_in_buffer = if cursor_x > buffer_len as u16 {
//             0 // Clamp to start if somehow cursor_x is too large
//         } else {
//             buffer_len - cursor_x as usize
//         };

//         // Total position in the terminal line (including prompt)
//         let total_cursor_pos = prompt_length as usize + cursor_position_in_buffer;

//         // Calculate which line and column the cursor should be on
//         let cursor_line_offset = total_cursor_pos / term_width as usize;
//         let cursor_col = (total_cursor_pos % term_width as usize) + 1;

//         let final_line = prompt_start_line + cursor_line_offset as u16;

//         // Update cursor position tracking
//         self.cursor_position.y = cursor_line_offset as u16;

//         // Move cursor to final position
//         Self::print_out_static(
//             &mut self.stdout,
//             &format!("{}", Goto(cursor_col as u16, final_line)),
//         );
//     }

//     // Static helper method to avoid borrowing issues
//     fn print_out_static(stdout: &mut Option<RawTerminal<Stdout>>, input: &str) {
//         match stdout {
//             Some(raw_stdout) => {
//                 write!(raw_stdout, "{}", input).unwrap();
//                 raw_stdout.flush().unwrap();
//             }
//             None => {
//                 let mut std = std::io::stdout();
//                 write!(std, "{}", input).unwrap();
//                 std.flush().unwrap();
//             }
//         }
//     }

//     pub fn insert_char(&mut self, c: char) {
//         let insert_pos = self.buffer.len() - self.cursor_position.x as usize;
//         self.buffer.insert(insert_pos, c);
//         self.rerender();
//     }

//     pub fn delete_char(&mut self) {
//         if self.buffer.is_empty() {
//             return;
//         }

//         if self.cursor_position.x == 0 {
//             // Delete at end of buffer (backspace from end)
//             self.buffer.pop();
//             self.rerender();
//         } else {
//             // Delete character before cursor position
//             let delete_pos = self.buffer.len() - self.cursor_position.x as usize;
//             if delete_pos > 0 {
//                 let char = self.buffer.remove(delete_pos - 1);
//                 // Don't change cursor_position.x since we deleted before the cursor
//             } else {
//                 return; // Nothing to delete
//             }
//             self.rerender();
//         }
//     }

//     pub fn move_cursor_left(&mut self) {
//         if self.cursor_position.x < self.buffer.len() as u16 {
//             self.cursor_position.x += 1;
//             self.rerender();
//         }
//     }

//     pub fn move_cursor_right(&mut self) {
//         if self.cursor_position.x > 0 {
//             self.cursor_position.x -= 1;
//             self.rerender();
//         }
//     }

//     pub fn load_history_prev(&mut self) {
//         // self.buffer
//         let prev_history = self.history.prev();
//         if prev_history != self.buffer {
//             clear_buff_ter(&mut self.stdout, self.buffer.clone());
//         }
//         if !prev_history.is_empty() {
//             self.buffer = prev_history;
//             self.cursor_position.reset();
//             self.rerender();
//         }
//     }

//     pub fn load_history_next(&mut self) {
//         let next_history = self.history.next();
//         if next_history != self.buffer {
//             clear_buff_ter(&mut self.stdout, self.buffer.clone());
//         }
//         self.buffer = next_history; // Empty string if no next history
//         self.cursor_position.reset();
//         self.rerender();
//     }

//     pub fn clear_screen(&mut self) {
//         self.buffer.clear();
//         self.cursor_position.reset();
//         Self::print_out_static(
//             &mut self.stdout,
//             &format!("{}{}", clear::All, cursor::Goto(1, 1)),
//         );
//         display_prompt(&mut self.stdout);
//     }

//     pub fn run(&mut self) {
//         display_prompt(&mut self.stdout);

//         let stdin = self.stdin.lock();

//         for key in stdin.keys() {
//             match key.unwrap() {
//                 // Execute command
//                 termion::event::Key::Char('\n') => {
//                     self.cursor_position.reset();
//                     Shell::parse_and_exec(&mut self.stdout, &mut self.buffer, &mut self.history);
//                 }

//                 // Tab completion (placeholder)
//                 termion::event::Key::Char('\t') => {
//                     // TODO: Implement tab completion
//                 }

//                 // Insert character
//                 termion::event::Key::Char(c) => {
//                     self.insert_char(c);
//                 }

//                 // Delete character
//                 termion::event::Key::Backspace => {
//                     self.delete_char();
//                 }

//                 // History navigation
//                 termion::event::Key::Up => {
//                     self.load_history_prev();
//                 }

//                 termion::event::Key::Down => {
//                     self.load_history_next();
//                 }

//                 // Cursor movement
//                 termion::event::Key::Left => {
//                     self.move_cursor_left();
//                 }

//                 termion::event::Key::Right => {
//                     self.move_cursor_right();
//                 }

//                 // Clear screen
//                 termion::event::Key::Ctrl('l') => {
//                     self.clear_screen();
//                 }

//                 // Exit
//                 termion::event::Key::Ctrl('d') => {
//                     Self::print_out_static(&mut self.stdout, "\r");
//                     return;
//                 }

//                 // Word deletion (placeholder)
//                 termion::event::Key::Ctrl('w') => {
//                     // TODO: Implement word deletion
//                 }

//                 // Interrupt (placeholder)
//                 termion::event::Key::Ctrl('c') => {
//                     // TODO: Send SIGINT
//                 }

//                 // Suspend (placeholder)
//                 termion::event::Key::Ctrl('z') => {
//                     // TODO: Send SIGTSTP
//                 }

//                 _ => {}
//             }
//         }
//     }
// }
