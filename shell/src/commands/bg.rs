use crate::{ShellCommand, envirement::ShellEnv};

pub struct Bg {
    args: Vec<String>,
    flags: Vec<String>,
    env: ShellEnv,
}
impl ShellCommand for Bg {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        Ok(())
    }
}