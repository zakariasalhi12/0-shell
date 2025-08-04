use std::io::{Error};

use crate::{ShellCommand};
use crate::envirement::ShellEnv;
pub struct Echo {
    args: Vec<String>,
}

impl Echo {
    pub fn new(args: Vec<String>) -> Self {
        Echo { args }
    }

}

impl ShellCommand for Echo {
    fn execute(&self, _env: &mut ShellEnv) -> std::io::Result<()> {
     
        println!("{}\r", self.args.join(" "));
        Ok(())
    }
}
