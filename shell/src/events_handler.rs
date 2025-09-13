use crate::envirement::ShellEnv;
use crate::features::history;
use crate::features::history::History;
use crate::lexer::tokenize::Tokenizer;
use crate::parser::*;
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

use crate::features::jobs::ProcessStatus;
use crate::shell_interactions::utils::parse_input;
use crate::shell_interactions::utils::*;
use crate::{exec::*, parser};

use std::io::*;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{self};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

#[derive(Clone, PartialEq)]
pub enum ShellMode {
    Interactive,
    NonInteractive,
    Command(String),
}

pub enum OutputTarget {
    Raw(Option<RawTerminal<Stdout>>),
    Stdout(Stdout),
    Null,
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
    pub env: Arc<Mutex<ShellEnv>>,
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
        let env = Arc::new(Mutex::new(ShellEnv::new()));
        start_reaper(env.clone());
        Self {
            stdin: stdin(),
            stdout: stdout,
            buffer: String::new(),
            env: env.clone(),
            history: history::History::new(),
            cursor_position_x: 0,
            cursor_position_y: 0,
            buffer_lines: 0,
            need_to_up: false,
            free_lines: 0,
            mode,
            cursor_position: CursorPosition::new(0, 0),
        }
    }

    // if the character == \0 remove the character from the buffer instead of add it

    pub fn cooked_mode(stdout: &mut OutputTarget) {
        if let OutputTarget::Raw(raw) = stdout {
            if let Some(raw_stdout) = raw {
                match raw_stdout.suspend_raw_mode() {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                };
            }
        }
    }

    pub fn raw_mode(stdout: &mut OutputTarget) {
        if let OutputTarget::Raw(raw) = stdout {
            if let Some(raw_stdout) = raw {
                match raw_stdout.activate_raw_mode() {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                };
            }
        }
    }

    pub fn parse_and_exec(
        stdout: &mut OutputTarget,
        buffer: &mut String,
        history: &mut History,
        env: &Arc<Mutex<ShellEnv>>,
    ) {
        match stdout {
            OutputTarget::Raw(raw) => match raw {
                Some(s) => {
                    match writeln!(s) {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(1);
                        }
                    };
                    print!("\r\x1b[2K");
                    match s.flush() {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(1);
                        }
                    };
                }
                None => {}
            },
            OutputTarget::Stdout(stdout) => match stdout.flush() {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            },
            OutputTarget::Null => {}
        }

        if !buffer.trim().is_empty() {
            history.save(buffer.clone());
            Shell::cooked_mode(stdout);

            // Properly lock and use the environment
            {
                let mut shell = env.lock().unwrap_or_else(|e| e.into_inner());
                parse_input(&buffer, &mut *shell);
            }

            Shell::raw_mode(stdout);
        }

        buffer.clear();
        let std: &mut Option<RawTerminal<std::io::Stdout>> = match stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
            _ => {
                return;
            }
        };
        display_promt(std);

        // Properly lock and use the environment for reaping children
        {
            let mut shell = env.lock().unwrap_or_else(|e| e.into_inner());
            reap_children(&mut *shell);
        }
    }

    pub fn run_interactive_shell(&mut self) {
        let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
            OutputTarget::Raw(std) => std,
            OutputTarget::Stdout(_) => &mut None,
            _ => return,
        };
        display_promt(stdout);

        let stdin = self.stdin.lock();
        for key in stdin.keys() {
            let new_key = match key {
                Ok(val) => val,
                Err(e) => {
                    eprint!("{e}");
                    std::process::exit(0);
                }
            };

            match new_key {
                termion::event::Key::Char('\n') => {
                    self.cursor_position.reset();
                    Shell::parse_and_exec(
                        &mut self.stdout,
                        &mut self.buffer,
                        &mut self.history,
                        &self.env, // Pass Arc<Mutex<ShellEnv>> instead of trying to lock it here
                    );
                }
                termion::event::Key::Char('\t') => {
                    // TODO: Tab completion
                }
                termion::event::Key::Char(c) => {
                    self.insert_char(c);
                }
                termion::event::Key::Backspace => {
                    self.delete_char();
                }
                termion::event::Key::Up => {
                    self.load_history_prev();
                }
                termion::event::Key::Down => {
                    self.load_history_next();
                }
                termion::event::Key::Left => {
                    self.move_cursor_left();
                }
                termion::event::Key::Right => {
                    self.move_cursor_right();
                }
                termion::event::Key::Ctrl('l') => {
                    self.clear_screen();
                }
                termion::event::Key::Ctrl('d') => {
                    let stdout: &mut Option<RawTerminal<std::io::Stdout>> = match &mut self.stdout {
                        OutputTarget::Raw(std) => std,
                        OutputTarget::Stdout(_) => &mut None,
                        _ => return,
                    };
                    Self::print_out_static(stdout, "\r");
                    return;
                }
                termion::event::Key::Ctrl('c') => {
                    self.ctrl();
                    continue;
                }
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
            let line = match line {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            match Tokenizer::new(&line).tokenize() {
                Ok(tokens) => match parser::Parser::new(tokens).parse() {
                    Ok(ast) => match ast {
                        Some(tree) => {
                            // Properly lock the mutex before using it
                            let mut env_guard = self.env.lock().unwrap_or_else(|e| e.into_inner());
                            match execute(&tree, &mut *env_guard) {
                                Ok(status) => {
                                    env_guard.last_status = status;
                                }
                                Err(err) => {
                                    eprintln!("{}", err);
                                }
                            }
                        }
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
                    Some(tree) => {
                        // Properly lock the mutex before using it
                        let mut env_guard = self.env.lock().unwrap_or_else(|e| e.into_inner());
                        match execute(&tree, &mut *env_guard) {
                            Ok(status) => {
                                env_guard.last_status = status;
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        }
                    }
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

fn start_reaper(env: Arc<Mutex<ShellEnv>>) {
    thread::spawn(move || {
        loop {
            {
                let mut env_guard = match env.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                reap_children(&mut *env_guard);
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

fn reap_children(env: &mut crate::envirement::ShellEnv) {
    loop {
        match waitpid(
            None,
            Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED),
        ) {
            Ok(WaitStatus::Signaled(pid, _, _)) => {
                // Update the specific process status and check if job should be removed
                let should_remove = env
                    .jobs
                    .update_process_status(pid, ProcessStatus::Terminated);
                if should_remove {
                    // Find the job and remove it only if all processes are finished
                    if let Some(job) = env.jobs.find_job_by_any_pid(pid) {
                        let pgid = job.pgid;
                        env.jobs.remove_job(pgid);
                    }
                }
            }
            Ok(WaitStatus::Exited(pid, _)) => {
                // Update the specific process status and check if job should be removed
                let should_remove = env.jobs.update_process_status(pid, ProcessStatus::Done);
                if should_remove {
                    // Find the job and remove it only if all processes are finished
                    if let Some(job) = env.jobs.find_job_by_any_pid(pid) {
                        let pgid = job.pgid;
                        env.jobs.remove_job(pgid);
                    }
                }
            }
            Ok(WaitStatus::Stopped(pid, _)) => {
                println!();
                env.jobs.update_process_status(pid, ProcessStatus::Stopped);
            }
            Ok(WaitStatus::Continued(pid)) => {
                env.jobs.update_process_status(pid, ProcessStatus::Running);
            }
            Ok(WaitStatus::StillAlive) => break,
            Ok(WaitStatus::PtraceEvent(_, _, _)) | Ok(WaitStatus::PtraceSyscall(_)) => {
                // ignore these for normal shell
            }
            Err(nix::errno::Errno::ECHILD) => break,
            Err(_) => break,
        }
    }
}
