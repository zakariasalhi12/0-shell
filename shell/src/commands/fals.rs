use crate::{ShellCommand, envirement::ShellEnv, error::ShellError};

#[derive(Debug, PartialEq, Eq)]

// Todo : Need env implementation
pub struct False {
    pub args: Vec<String>,
}

impl False {
    pub fn new(args: Vec<String>) -> Self {
        False { args: args }
    }
}

impl ShellCommand for False {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        Ok(1)
    }
}
