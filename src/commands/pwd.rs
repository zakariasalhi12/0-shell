use crate::ShellCommand;

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
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
