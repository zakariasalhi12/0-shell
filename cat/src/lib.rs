// use crate::ShellCommand;
use std::fs::{File, canonicalize};
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
    pub fn execute(&self) -> std::io::Result<()> {
        println!("args {:?}", self.args.clone());
        if self.args.len() != 1 {
            for file in &self.args[1..] {
                // println!("{:?}", file);
                let file_path = canonicalize(file)?;
                let mut file_handle = File::open(&file_path)?;
                let content = read_to_string(&mut file_handle)?;
                // println!("{}\r", content);
                // println!("{:?}", content);
            }
        } else {
            // let stdin = stdin();
            // let mut stdout = stdout().into_raw_mode().unwrap();
            // let mut buffer = String::new();
            // for key in stdin.keys() {
            //     match key.unwrap() {
            //         // Parse Input
            //         termion::event::Key::Char('\n') => {
            //             writeln!(stdout, "\r").unwrap();
            //             writeln!(stdout, "{}\r", buffer).unwrap();
            //             stdout.flush().unwrap();
            //             buffer = String::new();
            //             continue;
            //         }

            //         termion::event::Key::Char(c) => {
            //             write!(stdout, "{}", c).unwrap();
            //             stdout.flush().unwrap();
            //             buffer.push(c);
            //         }

            //         termion::event::Key::Ctrl('d') => {
            //             write!(stdout, "\r").unwrap();
            //             stdout.flush().unwrap();
            //             return Ok(());
            //         }

            //         termion::event::Key::Ctrl('c') => {
            //             return Ok(());
            //         }
            //         _ => {}
            //     }
            //     stdout.flush().unwrap();
            // }
        }
        Ok(())
    }
}
