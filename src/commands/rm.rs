use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Rm {
    pub args: Vec<String>,
    pub opts: Vec<String>,
}

impl Rm {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        Rm { args, opts }
    }
}

impl ShellCommand for Rm {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
