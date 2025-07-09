use crate::ShellCommand;
use libc::statvfs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Index;
use std::path::PathBuf;
use std::{
    env, fs,
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
        let current = match env::current_dir() {
            Ok(val) => val,
            Err(..) => return Err("Error in current directory"),
        };
        if !is_writable(current.join(dest))
            || !source.iter().any(|file| is_readable(current.join(dest)))
        {
            return Err("");
        }
        self.args.iter().enumerate().for_each(|(index, arg)| {
            if index != self.args.len() - 1 {
                println!("{}", index);
                source.push(&arg);
            }
        });
        dest = &self.args[self.args.len() - 1];
        Ok((source, dest))
    }
}

// fn Check_permision(source: &PathBuf , dest: &PathBuf) -> bool {
//     if source.metadata().p
// }

fn is_writable(path: &PathBuf) -> bool {
    OpenOptions::new().write(true).open(path).is_ok()
}

fn is_readable(path: &PathBuf) -> bool {
    File::open(path).is_ok()
}
// 🟢 2. Path Checks
//  Check that source exists.
// Use filesystem metadata functions (std::fs::metadata  equivalent).
//  Determine if source is:
// - File
// - Directory
// - Symlink
// - Check if target exists.
// If it is a directory:
// Plan to move into the directory (target/source_basename).

// 🟢 3. Permissions
//  Confirm read permissions on source.
//  Confirm write permissions in the destination directory.
//  Confirm you can remove the original after moving.

// 🟢 4. Same Filesystem vs Cross-Filesystem
//  Detect whether source and target are on the same filesystem.
// (e.g., using device IDs from metadata if you want to be robust)
//  If same filesystem:
//  Perform rename operation (fast).
//  If different filesystems:
//  Copy the file/directory recursively.
//  Preserve permissions and timestamps.
//  Delete the original after successful copy.

// 🟢 5. Copying Logic (Cross-Filesystem)
//  For files:
// Read data and write to target.
// Copy permissions, timestamps.
//  For directories:
// Recursively create directories.
// Recursively copy contents.
// Preserve metadata.
//  For symlinks:
// Recreate the symlink pointing to the same target.

// 🟢 6. Cleanup and Error Handling
//  If copying fails, clean up partial copies.
//  If deleting fails after copying, report error but keep the copy.
//  Handle special cases (moving onto itself, etc.).

// 🟢 7. User Feedback
//  Print clear error messages on:
// Missing files
// Permission errors
// Invalid arguments

//  Optionally support -v (verbose) flag for output.
// 🟢 8. Testing
//  Test moving a file in the same directory (rename).
//  Test moving to another directory on the same filesystem.
//  Test moving to another filesystem.
//  Test moving directories.
//  Test moving symlinks.
//  Test permission errors.
//  Test moving onto an existing target.

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
            Err(..) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Error: Invalid Parameters",
                ));
            }
        };
        println!("{:?} {}", source, dest);
        Ok(())
    }
}
