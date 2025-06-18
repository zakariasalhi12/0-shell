use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Rm {
    pub args: Vec<String>,
}

impl Rm {
    pub fn new(args: Vec<String>) -> Self {
        Rm { args: args }
    }
}

impl ShellCommand for Rm {
    fn execute(&self) {}
}
