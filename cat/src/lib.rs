use std::fs::{canonicalize, File};
use std::io::*;
#[derive(Debug, PartialEq, Eq)]
pub struct Cat {
    pub args: Vec<String>,
}

impl Cat {
    pub fn new(args: Vec<String>) -> Self {
        Cat { args: args }
    }
    pub fn execute(&self) -> std::io::Result<()> {
        if self.args.len() != 0 {
            for file in &self.args {
                let file_path = canonicalize(file)?;
                let mut file_handle = File::open(&file_path)?;
                let content = read_to_string(&mut file_handle)?;
                println!("{}\r", content);
            }
        } else {
            let stdin = std::io::stdin();
            let mut stdout = std::io::stdout();

            let stdin_lock = stdin.lock();

            for line in stdin_lock.lines() {
                let line = line?;
                writeln!(stdout, "{}\r", line)?;
                stdout.flush()?;
            }
        }
        Ok(())
    }
}
