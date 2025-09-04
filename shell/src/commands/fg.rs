use crate::{ShellCommand, envirement::ShellEnv};

pub struct Fg {
    args: Vec<String>,
    // env: ShellEnv,
}

impl Fg {
    pub fn new(args: Vec<String>) -> Self {
        Self {args}
    }
}


impl ShellCommand for Fg {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        Ok(())
    }
}