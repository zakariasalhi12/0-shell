use std::io::{Error, ErrorKind, Result};

pub use crate::ShellCommand;
use crate::config::ENV;

pub struct Export {
    pub args: Vec<String>,
}

impl Export {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl ShellCommand for Export {
    fn execute(&self) -> Result<()> {
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

            let env_result = ENV.lock();
            if let Ok(mut env_map) = env_result {
                env_map.insert(key.to_string(), value.to_string());
            } else {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to acquire ENV lock",
                ));
            }
        }

        Ok(())
    }
}
