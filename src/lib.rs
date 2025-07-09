mod commands;
use commands::*;
pub struct Cmd {
    pub input: String,
    pub empty: bool,
}

impl Cmd {
    pub fn new(input: String) -> Self {
        Self{input: input.trim().to_string(), empty: false}
    }
    pub fn parse_exec(&self) {
        if self.empty {
            return
        }
       let command = self.input.split_whitespace().collect::<Vec<&str>>();
       let mut args = command;
       if args.len() > 0 {
       let command = args.remove(0);
        match command {
            command if command == "echo" => echo(args),
            command if command == "cd" => cd(),
            command if command == "ls" => ls(),
            command if command == "pwd" => pwd(),
            command if command == "cat" => cat(),
            command if command == "cp" => cp(),
            command if command == "rm" => rm(),
            command if command == "mv" => mv(),
            command if command == "mkdir" => mkdir(),
            command if command == "exit" => exit(),
            _ => println!("Command '<{command}>' not found"),
        }
    }
    }
}


