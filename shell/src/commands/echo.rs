use nix::unistd::dup;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::OwnedFd;
use std::os::unix::io::{AsRawFd, FromRawFd};
use crate::error::ShellError;

use crate::ShellCommand;
use crate::envirement::ShellEnv;
pub struct Echo {
    args: Vec<String>,
    stdout: Option<OwnedFd>,
}

impl Echo {
    pub fn new(args: Vec<String>, stdout: Option<OwnedFd>) -> Self {
        Echo { args, stdout }
    }
}

fn interpret_escapes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('v') => result.push('\u{000B}'),
                Some('a') => result.push('\u{0007}'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

impl ShellCommand for Echo {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        let joined = self.args.join(" ");
        let output = interpret_escapes(&joined) + "\n";

        match &self.stdout {
            Some(raw_stdout) => {
                let fd =  match dup(raw_stdout.as_raw_fd()){
                    Ok(fd) => fd,
                    Err(e) => {
                        return Err(ShellError::Exec(e.desc().to_owned()))
                    }
                }; // duplicate to avoid closing original
                let mut file = unsafe { File::from_raw_fd(fd) };
                write!(file, "{}", output)?;
                file.flush()?;
            }
            None => {
                let mut std = io::stdout();
                write!(std, "{}", output)?;
                std.flush()?;
            }
        }

        Ok(0)
    }
}
