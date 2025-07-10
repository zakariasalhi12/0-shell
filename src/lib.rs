mod commands;
use commands::*;
pub struct Cmd {
    pub input: String,
    pub empty: bool,
}

fn parse_word(s: String) -> Result<String, ()> {
    let mut word = String::new();
    let mut inside = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, inside) {
            ('\"', None) => inside = Some('\"'),
            ('\"', Some('\"')) => inside = None,

            ('\\', Some('\"')) => {
                if let Some(n_c) = chars.next() {
                    if matches!(n_c, '\"') {
                        word.push(n_c);
                        continue;
                    }
                    word.push('\\');
                    word.push(n_c);
                } else {
                    return Err(());
                }
            }
            _ => word.push(c),
        }
    }
    if inside.is_some() {
        return Err(());
    }
    Ok(word)
}

fn parse_quotes(s: Vec<String>) -> Result<Vec<String>, ()> {
    let mut res: Vec<String> = vec![];
    let mut i = 0;
    println!("{:?}", s);
    while i < s.len() {
        if let Ok(word) = parse_word(s[i].clone()) {
            res.push(word);
        } else {
            return Err(());
        }
        i += 1;
    }
    Ok(res)
}

impl Cmd {
    pub fn new(input: String) -> Self {
        Self {
            input: input.trim().to_string(),
            empty: false,
        }
    }
    pub fn parse_exec(&self) {
        if self.empty {
            return;
        }
        let command = self
            .input
            .split_whitespace()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        let mut args = command;
        if args.len() > 0 {
            let command = args.remove(0);
            if let Ok(arg) = parse_quotes(args) {
                match command {
                    commmand if command == "echo" => echo(arg),
                    command if command == "cd" => cd(),
                    command if command == "ls" => ls(),
                    command if command == "pwd" => pwd(),
                    command if command == "cat" => cat(),
                    command if command == "cp" => cp(),
                    command if command == "rm" => rm(),
                    command if command == "mv" => mv(),
                    command if command == "mkdir" => mkdir(arg),
                    command if command == "exit" => exit(),
                    _ => println!("Command '<{command}>' not found"),
                }
            } else {
                println!("parse err!");
            }
        }
    }
}
