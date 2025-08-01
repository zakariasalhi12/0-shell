use nix::unistd::dup;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::{
    io::{Error, Stdout},
    os::fd::OwnedFd,
};

use crate::ShellCommand;
pub struct Echo {
    args: Vec<String>,
    stdout: Option<OwnedFd>,
}

impl Echo {
    pub fn new(args: Vec<String>, stdout: Option<OwnedFd>) -> Self {
        Echo { args, stdout }
    }
    pub fn format_input(&self) -> Option<Vec<String>> {
        let input = self.args.join(" ");
        let mut res: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = input.chars().peekable();

        let mut quote: Option<char> = None;
        let mut escaped = false;

        while let Some(c) = chars.next() {
            if escaped {
                current.push(c);
                escaped = false;
                continue;
            }

            match c {
                '\\' => {
                    escaped = true;
                }
                '\'' | '"' => {
                    if let Some(q) = quote {
                        if c == q {
                            quote = None;
                            res.push(current.clone());
                            current.clear();
                        } else {
                            current.push(c);
                        }
                    } else {
                        quote = Some(c);
                    }
                }
                ' ' if quote.is_none() => {
                    if !current.is_empty() {
                        res.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if escaped || quote.is_some() {
            None // unclosed quote or trailing escape
        } else {
            if !current.is_empty() {
                res.push(current);
            }
            Some(res)
        }
    }
}

impl ShellCommand for Echo {
    fn execute(&self) -> io::Result<()> {
        let text = match self.format_input() {
            Some(val) => val,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid format quotes",
                ));
            }
        };

        let output = text.join(" ") + "\n";

        match &self.stdout {
            Some(raw_stdout) => {
                let fd = dup(raw_stdout.as_raw_fd())?; // duplicate to avoid closing original
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

        Ok(())
    }
}
