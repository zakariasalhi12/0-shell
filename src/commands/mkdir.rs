use std::cell::RefCell;
use std::{fs, path::PathBuf};

use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct mkdir {
    pub args: Vec<String>,
    pub flags: Vec<String>,
}

impl mkdir {
    pub fn new(args: Vec<String>, flags: Vec<String>) -> Self {
        mkdir { args: args, flags }
    }
    pub fn parse_flags(&self) -> bool {
        for flag in &self.flags {
            if flag == "-p" {
                return true;
            }
        }
        false
    }
}

impl ShellCommand for mkdir {
    fn execute(&self) -> std::io::Result<()> {
        let is_parent = self.parse_flags();

        for drc in &self.args {
            let path = PathBuf::from(drc);

            if is_parent {
                fs::create_dir_all(&path)?;
            } else {
                if path.exists() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!("mkdir: cannot create directory '{}': File exists", drc),
                    ));
                }
                fs::create_dir(&path)?;
            }
        }
        Ok(())
    }
}
