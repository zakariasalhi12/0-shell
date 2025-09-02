use crate::{ShellCommand, commands::jobs, envirement::ShellEnv};

pub struct Jobs {
    args: Vec<String>,
    flags: Vec<String>,
    env: ShellEnv,
}

impl ShellCommand for Jobs {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        Ok(())
    }
}
