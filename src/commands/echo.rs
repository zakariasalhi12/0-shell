use std::io::{Error, stdout};

use crate::{ShellCommand, events_handler::print_out};
pub struct Echo {
    args: Vec<String>,
}

impl Echo {
    pub fn new(args: Vec<String>) -> Self {
        Echo { args }
    }
    pub fn format_input(&self) -> Option<Vec<String>> {
        let input = self.args.join(" ");
        let mut res: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = input.chars().peekable();

        let mut quote: Option<char> = None;
        let mut escaped = false;

        while let Some(c) = chars.next() {
            if escaped {
                current.push(c);
                escaped = false;
                continue;
            }

            match c {
                '\\' => {
                    escaped = true;
                }
                '\'' | '"' => {
                    if let Some(q) = quote {
                        if c == q {
                            quote = None;
                            res.push(current.clone());
                            current.clear();
                        } else {
                            current.push(c);
                        }
                    } else {
                        quote = Some(c);
                    }
                }
                ' ' if quote.is_none() => {
                    if !current.is_empty() {
                        res.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if escaped || quote.is_some() {
            None // unclosed quote or trailing escape
        } else {
            if !current.is_empty() {
                res.push(current);
            }
            Some(res)
        }
    }
}

impl ShellCommand for Echo {
    fn execute(&self) -> std::io::Result<()> {
        let text = match self.format_input() {
            Some(val) => val,
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalide format quotes",
                ));
            }
        };
        println!("{}\r", text.join(" "));
        Ok(())
    }
}
