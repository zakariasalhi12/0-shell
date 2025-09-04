use crate::{commands::jobs, features, ShellCommand};
use std::{
    io::{Error, ErrorKind},
};
use crate::envirement::ShellEnv;

pub struct Kill {
    args: Vec<String>,
}

impl Kill {
    pub fn new(args: Vec<String>) -> Self {
        Self {args}
    }
}

impl ShellCommand for Kill {
    fn execute(&self, env: &mut ShellEnv) -> std::io::Result<()> {
        let DefaultSignal = 15 ; // SIGTERM

        if self.args.len() < 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "kill: not enough arguments"));
        }

        // %1 for id

        println!("Killing process: {:?}", self.args);

        Ok(())
    }
}