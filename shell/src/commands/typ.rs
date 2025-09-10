use crate::envirement::ShellEnv;

use crate::error::ShellError;
use crate::ShellCommand;
use crate::exec::CommandType;

use crate::exec::get_command_type;

pub struct Type {
    pub args: Vec<String>,
}

impl Type {
    pub fn new(args: Vec<String>) -> Self {
        Type { args }
    }
}
impl ShellCommand for Type {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        if self.args.len() < 1  {
            return Err(ShellError::InvalidInput("type: missing operand".to_owned()));
        }else{
            let cmd = self.args[0].as_str();
        match  get_command_type(cmd, _env){
            CommandType::Builtin => println!("{} is a push Builtin\r", cmd),
            CommandType::External(path) =>println!("{} is an external command located at: {}\r", cmd, path),
            CommandType::Function(func) =>  println!("{} is a function with definition: {}\r", cmd, func),
            CommandType::Undefined => println!("{} is not a command\r", cmd),
        }
        return  Ok(0);
        }
    }
}