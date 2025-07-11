use crate::ShellCommand;
pub struct Echo {
    args: Vec<String>,
}

impl Echo {
    pub fn new(args: Vec<String>) -> Self {
        Echo { args }
    }
}

impl ShellCommand for Echo {
    fn execute(&self) -> std::io::Result<()> {
        let text = self.args.join(" ");
        print!("{text}\n\r");
        Ok(())
    }
}
