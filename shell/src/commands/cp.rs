use crate::ShellCommand;
use std::path::{Path, PathBuf};
use std::{
    env, fs,
    io::{Error, ErrorKind},
};

use crate::envirement::ShellEnv;


#[derive(Debug, PartialEq, Eq)]
pub struct Cp {
    pub args: Vec<String>,
    pub flags: Vec<String>,
}

impl Cp {
    pub fn new(args: Vec<String>, flags: Vec<String>) -> Self {
        Cp { args, flags }
    }

    fn validate_args(&self) -> bool {
        self.args.len() >= 2
    }

    fn get_param(&self) -> (Vec<String>, String) {
        let sources = self.args[..self.args.len() - 1].to_vec();
        let dest = self.args[self.args.len() - 1].clone();
        (sources, dest)
    }

    pub fn check_is_rec(&self) -> bool {
        self.flags.iter().any(|f| f == "-r" || f == "-R")
    }
}

/// Recursively copies a directory
pub fn copy_directory(src: &Path, dest: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Source is not a directory",
        ));
    }
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_directory(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}

impl ShellCommand for Cp {
    fn execute(&self, _env: &mut ShellEnv) -> std::io::Result<()> {
        if !self.validate_args() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "cp: missing file operand",
            ));
        }

        let (sources, dest) = self.get_param();
        let dest_path = PathBuf::from(&dest);
        let dest_is_dir = dest_path.is_dir() || (sources.len() > 1);

        for src_str in sources {
            let src_path = PathBuf::from(&src_str);

            if !src_path.exists() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("cp: cannot stat '{}': No such file or directory", src_str),
                ));
            }

            let target_path = if dest_is_dir {
                dest_path.join(src_path.file_name().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        "cp: invalid source path without filename",
                    )
                })?)
            } else {
                dest_path.clone()
            };

            if src_path.is_dir() {
                if self.check_is_rec() {
                    copy_directory(&src_path, &target_path)?;
                } else {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("cp: -r not specified; omitting directory '{}'", src_str),
                    ));
                }
            } else {
                fs::copy(&src_path, &target_path)?;
            }
        }

        Ok(())
    }
}
