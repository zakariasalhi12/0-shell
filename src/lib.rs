mod commands;
use commands::*;
pub struct Cmd {
    pub input: String,
    pub empty: bool,
}


fn clean_quotes(s: Vec<String>) -> bool{
    let mut inside = false;
    let mut count = 0;
   s.chars().for_each(|c| if *c == '"' {count++});
    true
}
fn parse_quotes(s: String)-> String {
    let s = s.replace("\"\"", "\" \"");
    s.split(" ").map(|c| c.trim_matches('"').to_string()).collect::<String>()
}

impl Cmd {
    pub fn new(input: String) -> Self {
        Self{input: input.trim().to_string(), empty: false}
    }
    pub fn parse_exec(&self) {
        if self.empty {
            return
        }
        let command = self.input.split_whitespace().map(|c| c.to_string()).collect::<Vec<String>>();
        
       let mut args = command;
       println!("{:?}", args);
       if args.len() > 0 {
        if clean_quotes(args){
       let command = args.remove(0);
       if args.len() > 0 {
            args = args.iter().map(|c|{ if c.starts_with("\"") {parse_quotes(c.to_string())} else {c.to_string()}}).collect::<Vec<String>>();
       }
        match command {
            commmand if command ==  "echo" => echo(args),
            command if command == "cd" => cd(),
            command if command == "ls" => ls(),
            command if command == "pwd" => pwd(),
            command if command == "cat" => cat(),
            command if command == "cp" => cp(),
            command if command == "rm" => rm(),
            command if command == "mv" => mv(),
            command if command == "mkdir" => mkdir(args),
            command if command == "exit" => exit(),
            _ => println!("Command '<{command}>' not found"),
        }
    }
       }
    }
}


