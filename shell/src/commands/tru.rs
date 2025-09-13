use crate::{ShellCommand, envirement::ShellEnv, error::ShellError};

#[derive(Debug, PartialEq, Eq)]

// Todo : Need env implementation
pub struct True {
    pub args: Vec<String>,
}

impl True {
    pub fn new(args: Vec<String>) -> Self {
        True { args: args }
    }
}

impl ShellCommand for True {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        Ok(0)
    }
}
