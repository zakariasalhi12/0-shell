use std::env;

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
    fn execute(&self, _env: &mut ShellEnv) -> std::io::Result<()> {
        let current = env::current_dir()?;
        println!(
            "{}\r",
            current
                .as_path()
                .to_str()
                .unwrap_or("Error: current Directory")
        );
        Ok(())
    }
}
