use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ShellError;
use crate::ShellCommand;
use crate::envirement::ShellEnv;

#[derive(Debug, PartialEq, Eq)]
pub struct Rm {
    pub args: Vec<String>,
    pub opts: Vec<String>,
    pub is_recursion: bool,
    pub valid_opts: bool,
}

impl Rm {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        let mut rs = Rm {
            args,
            opts,
            is_recursion: false,
            valid_opts: false,
        };
        rs.parse_flags();
        rs
    }

    pub fn parse_flags(&mut self) {
        for f in &self.args {
            if f.starts_with('-') {
                for ch in f.chars().skip(1) {
                    match ch {
                        'r' => self.is_recursion = true,
                        _ => {
                            self.valid_opts = false;
                            return;
                        }
                    }
                }
            }
        }
        self.args = self
            .args
            .iter()
            .filter(|f| !f.starts_with('-'))
            .cloned()
            .collect();
    }
}

fn delete_recursive(path: &Path) -> Result<i32, ShellError> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                delete_recursive(&entry_path)?;
            } else {
                fs::remove_file(&entry_path)?;
            }
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(0)
}

impl ShellCommand for Rm {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        if self.args.is_empty() {
            return Err(ShellError::InvalidInput("rm: missing operand".to_owned()));
        }
        let recursive = self.is_recursion;
        if self.opts.len() != 0 && !recursive {
            return Err(ShellError::InvalidInput(format!("{}: Invalid flag", self.opts[0])));
        }

        for target in &self.args {
            if target == "." || target == ".." {
                eprintln!("rm: refusing to remove '.' or '..' directory: skipping '..'\r");
                continue;
            }
            let path = PathBuf::from(target);
            if !path.exists() {
                eprintln!(
                    "rm: cannot remove '{}': No such file or directory\r",
                    target
                );
                continue;
            }

            if path.is_dir() {
                if recursive {
                    delete_recursive(&path)?;
                } else {
                    eprintln!("rm: cannot remove '{}': Is a directory\r", target);
                }
            } else {
                fs::remove_file(&path)?;
            }
        }
        Ok(0)
    }
}
