use std::io::{Error, ErrorKind, Result};

pub use crate::ShellCommand;
use crate::envirement::ShellEnv;
pub struct Export {
    pub args: Vec<String>,
}

impl Export {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl ShellCommand for Export {
    fn execute(&self, env: &mut ShellEnv) -> Result<()> {
        for arg in &self.args {
            let spliced: Vec<&str> = arg.splitn(2, '=').collect();

            if spliced.len() != 2 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid export syntax: '{}'", arg),
                ));
            }

            let key = spliced[0].trim_matches(|c| c == '\'' || c == '"');
            let value = spliced[1].trim_matches(|c| c == '\'' || c == '"');

  
                env.set_env_var(key, value);
           
        }

        Ok(())
    }
}
