use crate::ShellCommand;
use std::{fs, path::Path};
use std::io::{Error, ErrorKind};
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct Mkdir {
    pub args: Vec<String>,
}

impl Mkdir {
    pub fn new(args: Vec<String>) -> Self {
        Mkdir { args: args }
    }
}

impl ShellCommand for Mkdir {
    fn execute(&self) -> io::Result<()> {

        if self.args.len() == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mkdir: missing operand\nusage: mkdir test1 test2"));
        }

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