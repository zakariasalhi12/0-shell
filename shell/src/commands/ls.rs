use crate::ShellCommand;
use std::io::{ Result};
use std::process::Stdio;
use std::process::{ Command as ExternalCommand};

use crate::envirement::ShellEnv;

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
    fn execute(&self, _env: &mut ShellEnv) -> Result<()> {
        let mut child = ExternalCommand::new("/home/aelhadda/0-shell/bin/ls") // Use full_path here
            .args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
        Ok(())
    }
}
