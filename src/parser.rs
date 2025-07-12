use std::io::{stdout, Write};

use shell::ShellCommand;
use shell::commands::{
    cat::Cat, cd::Cd, cp::Cp, echo::Echo, ls::Ls, mkdir::Mkdir, mv::Mv, pwd::Pwd, rm::Rm
};

use crate::shell_handler::Shell;

// #[derive(Debug)]
pub struct Commande {
    pub operator: ExecType, //if this commande should run async (case of &) or sync (case of && or ; or | )
    pub name: Stdcommands, // "ls"
    pub option: Vec<String>, //"-f j"
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum ExecType {
    Sync,       // ; or nothing
    And,        // &&
    Or,         // ||
    Pipe,       // |
    Background, // &
}

#[derive(Debug)]
pub enum Stdcommands {
    ECHO,
    CD,
    LS,
    PWD,
    CAT,
    CP,
    RM,
    MV,
    MKDIR,
    EXIT,
}

impl Stdcommands {
    pub fn build_command(
        &self,
        args: Vec<String>,
        opts: Vec<String>,
    ) -> Option<Box<dyn ShellCommand>> {
        match self {
            Stdcommands::ECHO => Some(Box::new(Echo::new(args))),
            Stdcommands::CD => Some(Box::new(Cd::new(args))),
            Stdcommands::LS => Some(Box::new(Ls::new(args, opts))),
            Stdcommands::PWD => Some(Box::new(Pwd::new(args))),
            Stdcommands::CAT => Some(Box::new(Cat::new(args))),
            Stdcommands::CP => Some(Box::new(Cp::new(args))),
            Stdcommands::RM => Some(Box::new(Rm::new(args, opts))),
            Stdcommands::MV => Some(Box::new(Mv::new(args))),
            Stdcommands::MKDIR => Some(Box::new(Mkdir::new(args))),
            Stdcommands::EXIT => {
                std::process::exit(0) // need to clean befor exit;
            }
        }
    }
}
pub fn parse(input: &str) -> Vec<Commande> {
    use ExecType::*;

    let mut commandes: Vec<Commande> = vec![];
    let mut current_exec_type = Sync;

    // Regex for splitting by all valid execution delimiters
    let re = regex::Regex::new(r"(\s*&&\s*|\s*\|\|\s*|\s*\|\s*|\s*;\s*|\s*&\s*)").unwrap();
    let mut last_index = 0;

    for mat in re.find_iter(input) {
        let command_str = input[last_index..mat.start()].trim();
        let delimiter = mat.as_str().trim();
        last_index = mat.end();

        if let Some(cmd) = parse_command(command_str, current_exec_type) {
            commandes.push(cmd);
        }

        // Update exec_type based on the delimiter
        current_exec_type = match delimiter {
            "&&" => And,
            "||" => Or,
            "|" => Pipe,
            "&" => Background,
            ";" => Sync,
            _ => Sync,
        };
    }

    // Handle the last part (after last delimiter)
    let remaining = input[last_index..].trim();
    if !remaining.is_empty() {
        if let Some(cmd) = parse_command(remaining, current_exec_type) {
            commandes.push(cmd);
        }
    }

    commandes
}

pub fn parse_command(input: &str, exec_type: ExecType) -> Option<Commande> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let name = tokens[0];
    let cmd_type = matcher(name)?;

    let mut option: Vec<String> = vec![];
    let mut args = vec![];

    for token in &tokens[1..] {
        if token.starts_with("-") {
            // option.push_str(token);
            // option.push(' ');
            option.push((token).to_string());
        } else {
            let words = match parse_word(token.to_string()) {
                Ok(val) => val,
                Err(..) => return None,
            };
            args.push(words);
        }
    }

    Some(Commande {
        operator: exec_type,
        name: cmd_type,
        option: option,
        args: args,
    })
}

pub fn matcher(cmd: &str) -> Option<Stdcommands> {
    return match cmd {
        "echo" => Some(Stdcommands::ECHO),
        "cd" => Some(Stdcommands::CD),
        "ls" => Some(Stdcommands::LS),
        "pwd" => Some(Stdcommands::PWD),
        "cat" => Some(Stdcommands::CAT),
        "cp" => Some(Stdcommands::CP),
        "rm" => Some(Stdcommands::RM),
        "mv" => Some(Stdcommands::MV),
        "mkdir" => Some(Stdcommands::MKDIR),
        "exit" => Some(Stdcommands::EXIT),
        _ => None,
    };
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
