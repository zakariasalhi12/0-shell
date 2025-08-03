use std::{fs, path::PathBuf};
use crate::ShellCommand;
use crate::envirement::ShellEnv;

#[derive(Debug, PartialEq, Eq)]
pub struct Mkdir {
    pub args: Vec<String>,
    pub flags: Vec<String>,
}

impl Mkdir {
    pub fn new(args: Vec<String>, flags: Vec<String>) -> Self {
        Mkdir { args: args, flags }
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

impl ShellCommand for Mkdir {
    fn execute(&self, _env: &mut ShellEnv) -> std::io::Result<()> {
        let is_parent = self.parse_flags();

        for drc in &self.args {
            let path = PathBuf::from(drc);

            if is_parent {
                fs::create_dir_all(&path)?;
            } else {
                if path.exists() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!("Mkdir: cannot create directory '{}': File exists", drc),
                    ));
                }
                fs::create_dir(&path)?;
            }
        }
        Ok(())
    }
}
