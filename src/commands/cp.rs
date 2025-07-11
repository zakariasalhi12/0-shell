use crate::{ShellCommand};

#[derive(Debug, PartialEq, Eq)]
pub struct Cp {
    pub args: Vec<String>,
}

impl Cp {
    pub fn new(args: Vec<String>) -> Self {
        Cp { args: args }
    }
}

impl ShellCommand for Cp {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
