use crate::{ShellCommand, envirement::ShellEnv};

pub struct Kill {
    args: Vec<String>,
    flags: Vec<String>,
    env: ShellEnv,
}

impl ShellCommand for Kill {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        Ok(())
    }
}
