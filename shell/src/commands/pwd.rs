use std::env;

use crate::error::ShellError;
use crate::ShellCommand;
use crate::envirement::ShellEnv;

#[derive(Debug, PartialEq, Eq)]

// Todo : Need env implementation
pub struct Pwd {
    pub args: Vec<String>,
}

impl Pwd {
    pub fn new(args: Vec<String>) -> Self {
        Pwd { args: args }
    }
}

impl ShellCommand for Pwd {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        let current = env::current_dir()?;
        println!(
            "{}\r",
            current
                .as_path()
                .to_str()
                .unwrap_or("Error: current Directory")
        );
        Ok(0)
    }
}
