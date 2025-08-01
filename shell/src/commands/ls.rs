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
    pub all: bool,
    pub classify: bool,
    pub format: bool,
    pub valid_opts: bool,
}

impl Ls {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        let mut res = Ls {
            args,
            opts,
            all: false,
            classify: false,
            format: false,
            valid_opts: true,
        };
        res.parse_flags();
        res
    }

    pub fn parse_flags(&mut self) {
        for f in &self.opts {
            if f.starts_with('-') && f.len() > 1 {
                for ch in f.chars().skip(1) {
                    match ch {
                        'a' => self.all = true,
                        'F' => self.classify = true,
                        'l' => self.format = true,
                        _ => {
                            self.valid_opts = false;
                            return;
                        }
                    }
                }
            } else {
                self.valid_opts = false;
                return;
            }
        }
    }
}

impl ShellCommand for Ls {
    fn execute(&self) -> Result<()> {
        // let mut command = Command::new("/home/youzar-boot/0-shell/bin/ls");
        // for arg in &self.args {
        //     command.arg(arg);
        // }

        // match command.spawn().and_then(|mut child| child.wait()) {
        //     Ok(status) => {
        //         if !status.success() {
        //             Err(Error::new(ErrorKind::InvalidInput, "Error invalid"))
        //         } else {
        //             Ok(())
        //         }
        //     }
        //     Err(e) => Err(Error::new(ErrorKind::InvalidInput, "Error invalid")),
        // }
        let mut child = ExternalCommand::new("/home/youzar-boot/0-shell/bin/ls") // Use full_path here
            .args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
        Ok(())
    }
}
