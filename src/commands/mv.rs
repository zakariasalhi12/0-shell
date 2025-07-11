use crate::ShellCommand;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::{
    self, env, fs,
    io::{Error, ErrorKind},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Mv {
    pub args: Vec<String>,
}

impl Mv {
    pub fn new(args: Vec<String>) -> Self {
        Mv { args: args }
    }
    fn validate_args(&self) -> bool {
        if self.args.len() < 2 {
            return false;
        }
        if self.args.len() > 2 {
            let destination = self.args[0].clone();
            let current = match env::current_dir() {
                Ok(val) => val,
                Err(..) => return false,
            };
            if !current.join(destination).is_dir() {
                return false;
            }
        }
        true
    }
    fn get_param(&self) -> Result<(Vec<&str>, &str), &str> {
        let mut source: Vec<&str> = vec![];
        let dest: &str;
        self.args.iter().enumerate().for_each(|(index, arg)| {
            if index != self.args.len() - 1 {
                source.push(&arg);
            }
        });
        dest = &self.args[self.args.len() - 1];
        Ok((source, dest))
    }
}

fn is_direc(path: &str) -> bool {
    let current = match env::current_dir() {
        Ok(val) => val,
        Err(..) => return false,
    };
    current.join(path).is_dir()
}

fn try_rename_or_copy(src: &str, dest: &str) -> std::io::Result<()> {
    let destination = match fs::canonicalize(dest) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    match fs::rename(src, destination) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::CrossesDevices => {
            fs::copy(src, dest)?;
            fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn copy_directory(src: &Path, dest: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        // Create the destination directory
        fs::create_dir_all(dest)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();
            let dest_path = dest.join(&entry_name);

            if entry_path.is_dir() {
                copy_directory(&entry_path, &dest_path)?;
            } else if entry_path.is_file() {
                fs::copy(&entry_path, &dest_path)?;
            } else {
                println!("Skipping {:?}", entry_path);
            }
        }
    } else if src.is_file() {
        fs::copy(src, dest)?;
    } else {
        return Err(Error::new(
            ErrorKind::Other,
            "Source is not file or directory",
        ));
    }

    Ok(())
}

impl ShellCommand for Mv {
    fn execute(&self) -> std::io::Result<()> {
        if !self.validate_args() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mv: missing file operand",
            ));
        }
        let (source, dest) = match self.get_param() {
            Ok(val) => val,
            Err(e) => {
                return Err(Error::new(ErrorKind::InvalidInput, e));
            }
        };
        if is_direc(dest) {
            for file in source {
                copy_directory(&fs::canonicalize(file)?, &fs::canonicalize(dest)?)?;
            }
        } else {
            match try_rename_or_copy(source[0], dest) {
                Ok(val) => return Ok(val),
                Err(e) => return Err(e),
            }
        }
        // println!("{:?} {}", source, dest);
        Ok(())
    }
}
