use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args: Vec<String>,
}

impl Ls {
    pub fn new(args: Vec<String>) -> Self {
        Ls { args: args }
    }
}

impl ShellCommand for Ls {
    fn execute(&self) {}
}
