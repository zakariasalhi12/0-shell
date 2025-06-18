use crate::ShellCommand;

#[derive(Debug, PartialEq, Eq)]
pub struct Mv {
    pub args: Vec<String>,
}

impl Mv {
    pub fn new(args: Vec<String>) -> Self {
        Mv { args: args }
    }
}

impl ShellCommand for Mv {
    fn execute(&self) {}
}
