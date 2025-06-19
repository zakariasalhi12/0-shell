use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct mkdir {
    pub args: Vec<String>,
}

impl mkdir {
    pub fn new(args: Vec<String>) -> Self {
        mkdir { args: args }
    }
}

impl ShellCommand for mkdir {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
