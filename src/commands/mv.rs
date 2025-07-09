use libc::statvfs;

use crate::ShellCommand;
use std::fs::File;
use std::io::Write;
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
}

// 🟢 2. Path Checks
//  Check that source exists.
// Use filesystem metadata functions (std::fs::metadata or equivalent).
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
        // if ther's more than 2 args the last arg should be a directory
        Ok(())
    }
}
