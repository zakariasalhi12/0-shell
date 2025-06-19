use crate::ShellCommand;

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
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
