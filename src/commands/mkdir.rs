use crate::ShellCommand;

use std::{fs, path::Path};
use std::io::ErrorKind;
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct mkdir {
    pub args: Vec<String>,
}

impl mkdir {
    pub fn new(args: Vec<String>) -> Self {
        mkdir { args: args }
    }
}

impl ShellCommand for mkdir {
    fn execute(&self) -> io::Result<()> {
        for arg in &self.args {
            let path = Path::new(arg);
            match fs::create_dir(path) {
                Ok(_) => continue,
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
