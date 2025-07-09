// use core::error;
use crate::ShellCommand;
use std::env;
use std::io::{Error, ErrorKind};

// use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct Cd {
    pub args: Vec<String>,
}

impl Cd {
    pub fn new(args: Vec<String>) -> Self {
        Cd { args: args }
    }
}

impl ShellCommand for Cd {
    fn execute(&self) -> std::io::Result<()> {
        let current_dir = env::current_dir()?;

        if self.args.len() != 1 {
            eprintln!(
                "cd: string not in pwd: {}",
                current_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("<unknown>")
            );
            return Err(Error::new(ErrorKind::NotFound, "No such file or directory"));
        }

        let dirname = current_dir.join(&self.args[0]);

        if !dirname.exists() {
            eprintln!(
                "cd: no such directory: {}",
                dirname.to_str().unwrap_or("<invalid path>")
            );
            return Err(Error::new(ErrorKind::NotFound, "Directory does not exist"));
        }

        env::set_current_dir(&dirname)?;
        Ok(())
    }
}
