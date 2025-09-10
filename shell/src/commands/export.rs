pub use crate::ShellCommand;
use crate::envirement::ShellEnv;
use crate::error::ShellError;
pub struct Export {
    pub args: Vec<String>,
}

impl Export {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl ShellCommand for Export {
    fn execute(&self, env: &mut ShellEnv) -> Result<i32, ShellError> {
        for arg in &self.args {
            if let Some(pos) = arg.find('=') {
                env.set_env_var(&arg[..pos], &arg[pos + 1..]);
            } else {
                if let Some(var) = env.get(&arg) {
                    env.set_env_var(&arg, &var);
                }
            }
        }
        Ok(0)
    }
}
