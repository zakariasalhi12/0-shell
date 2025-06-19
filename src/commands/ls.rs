use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args: Vec<String>,
    pub opts: Vec<String>,
}

impl Ls {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        Ls { args, opts }
    }
}

impl ShellCommand for Ls {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
