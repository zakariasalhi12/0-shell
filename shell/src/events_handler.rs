use crate::envirement::ShellEnv;
use crate::features::history;
use crate::features::history::History;
use crate::lexer::tokenize::Tokenizer;
use crate::parser::*;
use crate::shell_interactions::utils::clear_buff_ter;
use crate::{display_promt, prompt_len};
use crate::{exec::*, parser};
use std::io::*;
use std::io::{self, BufRead};
use std::{self};
use termion::cursor::DetectCursorPos;
use termion::cursor::Goto;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::{clear, cursor};
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub x: u16, // Position within the buffer (0 = at end)
    pub y: u16, // Line offset from prompt line
}

impl CursorPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

#[derive(Clone, PartialEq)]
pub enum ShellMode {
    Interactive,
    NonInteractive,
    Command(String),
}

pub enum OutputTarget {
    Raw(Option<RawTerminal<Stdout>>),
    Stdout(Stdout),
}

pub struct Shell {
    pub stdout: OutputTarget,
    pub stdin: Stdin,
    pub buffer: String,
    pub history: History,
    pub cursor_position_x: i16,
    pub cursor_position_y: u16,
    pub buffer_lines: u16,
    pub need_to_up: bool,
    pub free_lines: u16,
    pub env: ShellEnv,
    pub mode: ShellMode,
    pub cursor_position: CursorPosition,
}

impl Shell {
    pub fn new(mode: ShellMode) -> Self {
        let stdout = if mode == ShellMode::Interactive {
            match stdout().into_raw_mode() {
                Ok(raw) => OutputTarget::Raw(Some(raw)),
                Err(_) => {
                    eprintln!("no stdout");
                    std::process::exit(1);
                }
            }
        } else {
            OutputTarget::Stdout(stdout())
        };

        Self {
            stdin: stdin(),
            stdout: stdout,
            buffer: String::new(),
            history: history::History::new(),
            cursor_position_x: 0,
            cursor_position_y: 0,
            buffer_lines: 0,
            need_to_up: false,
            free_lines: 0,
            env: ShellEnv::new(),
            mode,
            cursor_position: CursorPosition::new(0, 0),
        }
    }

    pub fn rerender(&mut self) {
        let (term_width, _term_height) = termion::terminal_size().unwrap_or((80, 24));
        let prompt_length = prompt_len() as u16;
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
        };
        // Store values we need to avoid borrowing conflicts
        let buffer_clone = self.buffer.clone();
        let buffer_len = self.buffer.len();
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
            let (x, y) = std.cursor_pos().unwrap_or((1, 1));
            (x, y)
        } else {
            (1, 1)
        };
        // Calculate where the prompt starts
        let prompt_start_line = current_y.saturating_sub(cursor_y);

        // First, we need to clear all lines that might have old content
        // Go to the start of the prompt line

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

        // Calculate final cursor position
        // cursor_x represents how many characters from the END of buffer the cursor is
        // So if cursor_x = 0, cursor is at the very end
        // If cursor_x = 2, cursor is 2 characters before the end

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

    // if the character == \0 remove the character from the buffer instead of add it

    pub fn parse_and_exec(
        stdout: &mut OutputTarget,
        buffer: &mut String,
        history: &mut History,
        shell: &mut ShellEnv,
    ) {
        match stdout {
            OutputTarget::Raw(raw) => match raw {
                Some(s) => {
                    writeln!(s).unwrap();
                    print!("\r\x1b[2K");
                    s.flush().unwrap();
                }
                None => {}
            },
            OutputTarget::Stdout(stdout) => stdout.flush().unwrap(),
        }

        if !buffer.trim().is_empty() {
            history.save(buffer.clone());
            parse_input(&buffer, shell);
        }

        buffer.clear();
        let std: &mut Option<RawTerminal<std::io::Stdout>> = match stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
        };
        display_promt(std);
    }

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

    pub fn run_interactive_shell(&mut self) {
        // let stdin = &self.stdin;
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
        };

        display_promt(stdout);
        let stdin = self.stdin.lock();

        for key in stdin.keys() {
            match key.unwrap() {
                // Execute command
                termion::event::Key::Char('\n') => {
                    self.cursor_position.reset();
                    Shell::parse_and_exec(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.history,
                        &mut self.env,
                    );
                }

                // Tab completion (placeholder)
                termion::event::Key::Char('\t') => {
                    // TODO: Implement tab completion
                }

                // Insert character
                termion::event::Key::Char(c) => {
                    self.insert_char(c);
                }

                // Delete character
                termion::event::Key::Backspace => {
                    self.delete_char();
                }

                // History navigation
                termion::event::Key::Up => {
                    self.load_history_prev();
                }

                termion::event::Key::Down => {
                    self.load_history_next();
                }

                // Cursor movement
                termion::event::Key::Left => {
                    self.move_cursor_left();
                }

                termion::event::Key::Right => {
                    self.move_cursor_right();
                }

                // Clear screen
                termion::event::Key::Ctrl('l') => {
                    self.clear_screen();
                }

                // Exit
                termion::event::Key::Ctrl('d') => {
                    let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
                        OutputTarget::Raw(std) => std,
                        OutputTarget::Stdout(_) => &mut None,
                    };
                    Self::print_out_static(stdout, "\r");
                    return;
                }

                // Word deletion (placeholder)
                termion::event::Key::Ctrl('w') => {
                    // TODO: Implement word deletion
                }

                // Interrupt (placeholder)
                termion::event::Key::Ctrl('c') => {
                    // TODO: Send SIGINT
                }

                // Suspend (placeholder)
                termion::event::Key::Ctrl('z') => {
                    // TODO: Send SIGTSTP
                }

                _ => {}
            }
        }
    }
    fn print_out_static(stdout: &mut Option<RawTerminal<Stdout>>, input: &str) {
        match stdout {
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

    pub fn insert_char(&mut self, c: char) {
        let insert_pos = self.buffer.len() - self.cursor_position.x as usize;
        self.buffer.insert(insert_pos, c);
        self.rerender();
    }

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
        };
        self.buffer.clear();
        self.cursor_position.reset();
        Self::print_out_static(stdout, &format!("{}{}", clear::All, cursor::Goto(1, 1)));
        display_promt(stdout);
    }

    pub fn run_non_interactive_stdin(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            match Tokenizer::new(&line).tokenize() {
                Ok(tokens) => match parser::Parser::new(tokens).parse() {
                    Ok(ast) => match ast {
                        Some(tree) => match execute(&tree, &mut self.env) {
                            Ok(status) => {
                                self.env.last_status = status;
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        },
                        None => return,
                    },
                    Err(error) => {
                        eprintln!("{}", error,)
                    }
                },
                Err(error) => {
                    eprintln!("{}", error,)
                }
            }
        }
    }

    pub fn handle_command(&mut self, cmd: &str) {
        match Tokenizer::new(cmd).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(ast) => match ast {
                    Some(tree) => match execute(&tree, &mut self.env) {
                        Ok(status) => {
                            self.env.last_status = status;
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                        }
                    },
                    None => {
                        return;
                    }
                },
                Err(error) => {
                    eprintln!("{}", error,)
                }
            },
            Err(error) => {
                eprintln!("{}", error,)
            }
        };
    }

    pub fn run(&mut self) {
        match &self.mode {
            ShellMode::Interactive => self.run_interactive_shell(),
            ShellMode::NonInteractive => self.run_non_interactive_stdin(),
            ShellMode::Command(cmd) => self.handle_command(cmd.clone().as_str()),
        }
    }
}

pub fn print_out(w: &mut OutputTarget, input: &str) {
    match w {
        OutputTarget::Raw(raw_stdout) => match raw_stdout {
            Some(raw_stdout) => {
                write!(raw_stdout, "{}", input).unwrap();
                raw_stdout.flush().unwrap();
            }
            None => {
                eprintln!("raw stdout is not available");
            }
        },
        OutputTarget::Stdout(stdout) => {
            write!(stdout, "{}", input).unwrap();
            stdout.flush().unwrap();
        }
    }
}
pub fn parse_input(buffer: &str, mut env: &mut ShellEnv) {
    match Tokenizer::new(buffer.trim().to_owned().as_str()).tokenize() {
        Ok(res) => match Parser::new(res).parse() {
            Ok(ast) => match ast {
                Some(ast) => {
                    // println!("ast: {:?}", &ast);

                    match execute(&ast, &mut env) {
                        Ok(_status) => {
                            print!("\r");
                        }
                        Err(e) => eprintln!("{e}\r"),
                    }
                }
                None => println!("empty AST"),
            },
            Err(e) => {
                eprintln!("{:#?}", e);
            }
        },

        Err(err) => {
            eprintln!("{:#?}", err);
        }
    }
}
