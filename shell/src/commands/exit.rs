use crate::{error::ShellError, ShellCommand};
use crate::envirement::ShellEnv;

#[derive(Debug, PartialEq, Eq)]
pub struct Exit {
    pub args: Vec<String>,
    pub opts: Vec<String>,
}

impl Exit {
    pub fn new(mut args: Vec<String>, opts: Vec<String>) -> Self {
        if args.is_empty() {
            args.push("0".to_string());
        }
        Exit { args, opts }
    }
}

impl ShellCommand for Exit {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        if self.args.len() > 1 {
            return Err(ShellError::Exec(String::from("Exit command accepts at most one argument")));
        }
        let exit_code: i32 = self.args.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        std::process::exit(exit_code);
    }
}
