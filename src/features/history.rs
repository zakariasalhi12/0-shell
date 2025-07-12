use std::env::home_dir;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

pub struct History {
    pub path: String,
    pub history: Vec<String>,
    pub position: i32,
}

static NAME: &str = ".0-shell_history";

fn file_to_vec(path: String) -> Vec<String> {
    let mut file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(path)
        .unwrap();

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content.lines().map(|line| line.to_string()).collect()
}

impl History {
    pub fn new() -> Self {
        let file_path = format!(
            "{}/{}",
            home_dir().unwrap().to_string_lossy().to_owned(),
            NAME
        );

        let file_content = file_to_vec(file_path.to_owned());

        let history = History {
            path: file_path.to_owned(),
            position: file_content.len() as i32,
            history: file_content,
        };

        return history;
    }

    pub fn next(&mut self) -> String {
        if self.position - 1 < 0 {
            return "".to_owned();
        }
        self.position -= 1;
        return self.history[self.position as usize].to_owned();
    }

    pub fn prev(&mut self) -> String {
        if self.position + 1 >= self.history.len() as i32 {
            return "".to_owned();
        }
        self.position += 1;
        return self.history[self.position as usize].to_owned();
    }

    pub fn save(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.path.to_owned())
            .unwrap();

        file.write((command.to_string()  + "\n").as_bytes()).unwrap();
        self.history.push(command);
        self.position += 1;
    }

    pub fn listen(&mut self, buff: &mut String) {

    }

}
