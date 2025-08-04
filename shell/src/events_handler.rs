use crate::envirement::ShellEnv;
use crate::features::history;
use crate::features::history::History;
use crate::lexer::tokenize::Tokenizer;
use crate::parser::*;
use crate::shell_interactions::utils::clear_buff_ter;
use crate::shell_interactions::utils::parse_input;
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
