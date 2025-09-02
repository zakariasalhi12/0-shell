use crate::{ShellCommand, envirement::ShellEnv};

pub struct Fg {
    args: Vec<String>,
    flags: Vec<String>,
    env: ShellEnv,
}
impl ShellCommand for Fg {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        Ok(())
    }
}
