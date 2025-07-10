// use core::error;
use crate::ShellCommand;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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
        let target_dir: PathBuf;

        if self.args.is_empty() {
            // No argument: cd to home
            target_dir = env::home_dir()
                .ok_or(Error::new(ErrorKind::NotFound, "Home directory not found"))?;
        } else {
            let arg = self.args[0].as_str();
            if arg == "~" || arg.trim().is_empty() {
                target_dir = env::home_dir()
                    .ok_or(Error::new(ErrorKind::NotFound, "Home directory not found"))?;
            } else {
                target_dir = PathBuf::from(arg);
            }
        }

        if !target_dir.exists() {
            eprintln!(
                "cd: no such directory: {}",
                target_dir.to_str().unwrap_or("<invalid path>")
            );
            return Err(Error::new(ErrorKind::NotFound, "Directory does not exist"));
        }

        env::set_current_dir(&target_dir)?;
        Ok(())
    }
}
