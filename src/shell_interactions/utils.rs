use termion::{clear, cursor};
use termion::cursor::{DetectCursorPos, Goto, Left, Right, Up};
use termion::raw::RawTerminal;
use std::io::*;
use crate::{display_prompt, prompt_len, shell::*};


pub fn re_render(
    stdout: &mut Option<RawTerminal<Stdout>>,
    old_buffer: &mut String,
    new_buffer: String,
    cursor_position: &mut CursorPosition,
    del: bool,
) {
    if old_buffer.is_empty() || new_buffer.is_empty() {
        return;
    }

    let (x, y) = stdout.as_mut().unwrap().cursor_pos().unwrap_or((1, 1));
    let (width, height) = termion::terminal_size().unwrap_or((80, 24));

    let old_buffer_lines = calc_termlines_in_buffer(old_buffer.len());
    let new_buffer_lines = calc_termlines_in_buffer(new_buffer.len());

    let mut calc = (old_buffer_lines as i16 - new_buffer_lines as i16).abs();

    let free_lines = (height - y) - cursor_position.y;

    print_out(stdout, &format!("{}", Goto(1, height - free_lines))); // move the cursor to the last line

    for i in 0..calc_termlines_in_buffer(old_buffer.len()) {
        // clear all buffer lines
        if i > 0 {
            print_out(stdout, &format!("{}", Up(1)));
        }
        clear_current_line(stdout);
    }
    old_buffer.clear();
    display_prompt(stdout);
    if !del {
        cursor_position.y += calc as u16;
    } else {
        calc = 0;
    }
    print_out(
        stdout,
        &format!("{}{}", new_buffer, Goto(x, y - calc as u16)),
    ); // restore the old cursor position
}

// if the character == \0 remove the character from the buffer instead of add it
pub fn edit_buffer(
    stdout: &mut Option<RawTerminal<Stdout>>,
    character: char,
    buffer: &mut String,
    cursor_position: &mut CursorPosition,
) {
    let mut remove: i16 = 0;
    if character == '\0' {
        remove = -1
    }

    let mut res = String::new();
    for (i, c) in buffer.to_owned().char_indices() {
        if (i as i16) == (buffer.len() as i16) - cursor_position.x as i16 + remove {
            if character == '\0' {
                continue;
            }
            res.push(character);
        }
        res.push(c);
    }
    re_render(stdout, buffer, res.clone(), cursor_position, remove == -1);
    buffer.clear();
    if remove == -1 {
        print_out(stdout, &format!("{}", Left(1)));
    } else {
        print_out(stdout, &format!("{}", Right(1)));
    }
    buffer.push_str(&res);
}

pub fn clear_terminal(stdout: &mut Option<RawTerminal<Stdout>>, buffer: &mut String) {
    buffer.clear();
    print_out(stdout, &format!("{}{}\r", clear::All, cursor::Goto(1, 1)));
    display_prompt(stdout);
}

pub fn clear_current_line(stdout: &mut Option<RawTerminal<Stdout>>) {
    print_out(stdout, &format!("{}\r", clear::CurrentLine));
}

pub fn calc_termlines_in_buffer(buffer_size: usize) -> u16 {
    let (width, _) = termion::terminal_size().unwrap_or((80, 24));
    (width + ((buffer_size + prompt_len()) as u16 - 1)) / width
}

pub fn print_out(w: &mut Option<RawTerminal<Stdout>>, input: &str) {
    match w {
        Some(raw_stdout) => {
            write!(raw_stdout, "{}", input).unwrap();
            raw_stdout.flush().unwrap();
        }
        None => {
            let mut std = std::io::stdout();
            write!(std, "{}", input).unwrap();
            std.flush().unwrap();
        }
    }
}
