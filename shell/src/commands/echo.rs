use nix::unistd::dup;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::{
    io::{Error, Stdout},
    os::fd::OwnedFd,
};

use crate::envirement::ShellEnv;
use crate::ShellCommand;
pub struct Echo {
    args: Vec<String>,
    stdout: Option<OwnedFd>,
}

impl Echo {
    pub fn new(args: Vec<String>, stdout: Option<OwnedFd>) -> Self {
        Echo { args, stdout }
    }
}

impl ShellCommand for Echo {
    fn execute(&self, _env : &mut ShellEnv) -> io::Result<()> {
        let output = self.args.join(" ") + "\n";

        match &self.stdout {
            Some(raw_stdout) => {
                let fd = dup(raw_stdout.as_raw_fd())?; // duplicate to avoid closing original
                let mut file = unsafe { File::from_raw_fd(fd) };
                write!(file, "{}", output)?;
                file.flush()?;
            }
            None => {
                let mut std = io::stdout();
                write!(std, "{}", output)?;
                std.flush()?;
            }
        }

        Ok(())
    }
}
