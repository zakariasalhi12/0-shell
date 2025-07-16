use std::fs;
use std::path::{Path, PathBuf};

use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Rm {
    pub args: Vec<String>,
    pub opts: Vec<String>,
}

impl Rm {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        Rm { args, opts }
    }

    pub fn is_recursive(&self) -> bool {
        self.opts.iter().any(|f| f == "-r")
    }
}

fn delete_recursive(path: &Path) -> std::io::Result<()> {
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
    Ok(())
}

impl ShellCommand for Rm {
    fn execute(&self) -> std::io::Result<()> {
        if self.args.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "rm: missing operand",
            ));
        }

        let recursive = self.is_recursive();

        for target in &self.args {
            let path = PathBuf::from(target);
            if !path.exists() {
                eprintln!("rm: cannot remove '{}': No such file or directory\r", target);
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
        Ok(())
    }
}
