use crate::ShellCommand;
use std::io::*;
use std::process::Command;
use crate::envirement::ShellEnv;

#[derive(Debug, PartialEq, Eq)]
pub struct Cat {
    pub args: Vec<String>,
}

impl Cat {
    pub fn new(args: Vec<String>) -> Self {
        Cat { args: args }
    }
}

impl ShellCommand for Cat {
    fn execute(&self, _env: &mut ShellEnv) -> std::io::Result<()> {
        let mut command = Command::new("/home/aelhadda/0-shell/bin/cat");
        for arg in &self.args {
            command.arg(arg);
        }

        match command.spawn().and_then(|mut child| child.wait()) {
            Ok(status) => {
                if !status.success() {
                    Err(Error::new(ErrorKind::InvalidInput, "Error invalid"))
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(Error::new(ErrorKind::InvalidInput, "Error invalid")),
        }
    }
}
