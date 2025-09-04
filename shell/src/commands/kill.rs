use crate::{commands::jobs, features, ShellCommand};
use std::{
    io::{Error, ErrorKind},
};
use crate::envirement::ShellEnv;
use features::jobs::*;

pub struct Kill {
    args: Vec<String>,
    jobs: Jobs,
    // flags: Vec<String>,
    // env: ShellEnv,
}

impl Kill {
    pub fn new(args: Vec<String> , jobs : Jobs) -> Self {
        Self {args , jobs}
    }
}

impl ShellCommand for Kill {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        let DefaultSignal = 15 ; // SIGTERM

        if self.args.len() < 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "kill: not enough arguments"));
        }

        // %1 for id

        println!("Killing process: {:?}", self.args);

        Ok(())
    }
}