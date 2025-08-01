use crate::ShellCommand;
use std::fs::{FileType, Metadata};
use std::fs::{read_dir, symlink_metadata};
use std::io::Error;
use std::io::{ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::process::{Child, Command as ExternalCommand};
use std::time::UNIX_EPOCH;
use std::{self, fs};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args: Vec<String>,
    pub opts: Vec<String>,
}

impl Ls {
    pub fn new(mut args: Vec<String>, opts: Vec<String>) -> Self {
        args.extend(opts.iter().cloned()); // or just opts.clone()
        let res = Ls { args, opts };
        // res.parse_flags();
        res
    }
}

impl ShellCommand for Ls {
    fn execute(&self) -> Result<()> {
        let mut child = ExternalCommand::new("/home/yhajjaou/Desktop/0-shell/bin/ls") // Use full_path here
            .args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
        Ok(())
    }
}
