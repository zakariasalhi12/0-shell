use crate::ShellCommand;
use std::fs::{canonicalize, read};
use std::io::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Debug, PartialEq, Eq)]
pub struct Cat {
    pub args: Vec<String>,
}

impl Cat {
    pub fn new(args: Vec<String>) -> Self {
        Cat { args: args }
    }
}

impl ShellCommand for Cat {
    fn execute(&self) -> std::io::Result<()> {
        if self.args.len() != 0 {
            for file in &self.args {
                let file_name = canonicalize(file)?;
                let content = read(file_name)?;
                let content_str = String::from_utf8_lossy(&content);
                println!("{}", content_str);
            }
        } else {
            let stdin = stdin();
            let mut stdout = stdout().into_raw_mode().unwrap();
            let mut buffer = String::new();
            for key in stdin.keys() {
                match key.unwrap() {
                    // Parse Input
                    termion::event::Key::Char('\n') => {
                        writeln!(stdout).unwrap();
                        writeln!(stdout, "{}", buffer).unwrap();
                        stdout.flush().unwrap();
                        buffer = String::new();
                        continue;
                    }

                    termion::event::Key::Char(c) => {
                        write!(stdout, "{}", c).unwrap();
                        stdout.flush().unwrap();
                        buffer.push(c);
                    }

                    termion::event::Key::Ctrl('d') => {
                        write!(stdout, "\r").unwrap();
                        stdout.flush().unwrap();
                        return Ok(());
                    }

                    termion::event::Key::Ctrl('c') => {
                        return Ok(());
                    }
                    _ => {}
                }
                stdout.flush().unwrap();
            }
        }
        Ok(())
    }
}
