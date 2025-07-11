use std::env;
pub mod commands {
    pub mod cat;
    pub mod cd;
    pub mod cp;
    pub mod echo;
    pub mod ls;
    pub mod mkdir;
    pub mod mv;
    pub mod pwd;
    pub mod rm;
}

pub mod features {
    pub mod history;
}

enum Colors {
    // WHITE(String),
    // GREY(String),
    // BLUE(String),
    YELLOW(String),
}

impl Colors {
    fn to_ansi(&self) -> String {
        match self {
            // Colors::WHITE(text) => format!("\x1b[1;37m{}\x1b[0m", text),
            // Colors::GREY(text) => format!("\x1b[1;30m{}\x1b[0m", text),
            // Colors::BLUE(text) => format!("\x1b[1;34m{}\x1b[0m", text),
            Colors::YELLOW(text) => format!("\x1b[1;31m{}\x1b[0m", text),
        }
    }
}

pub fn get_first_element<'a>(s: &'a str, pattern: &str) -> &'a str {
    s.split(pattern).next().unwrap_or("")
}

pub fn get_current_directory() -> Result<String, String> {
    match env::current_dir() {
        Ok(path) => match path.file_name() {
            Some(name) => Ok(name.to_string_lossy().to_string()),
            None => Ok("/".to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn distplay_promt() {
    let current_directory = get_current_directory().unwrap();
    let prompt = Colors::YELLOW(format!("➜ {} ", current_directory));
    print!("{}", prompt.to_ansi());
}

pub trait ShellCommand {
    fn execute(&self) -> std::io::Result<()> {
        Ok(())
    }
}
