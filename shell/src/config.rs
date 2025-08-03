use dirs::home_dir;
use lazy_static::lazy_static;
use std::fs::read_to_string;
use std::{collections::HashMap, env, sync::Mutex};
use whoami;

fn get_user_shell(username: &str) -> Option<String> {
    read_to_string("/etc/passwd")
        .ok()?
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(6).map(String::from))
}

lazy_static! {
    pub static ref ENV: Mutex<HashMap<String, String>> = Mutex::new({
        let mut map = HashMap::new();

        // USER
        let username = whoami::username();
        map.insert("USER".to_string(), username.clone());

        // HOME and ~
        let home = home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .or_else(|| env::var("HOME").ok())
            .unwrap_or_else(|| "/".to_string());

        map.insert("HOME".to_string(), home.clone());
        map.insert("~".to_string(), home);

        // SHELL
        let shell = get_user_shell(&username).unwrap_or_default();
        map.insert("SHELL".to_string(), shell);

        // PWD (current directory)
        if let Ok(current_dir) = env::current_dir() {
            map.insert("PWD".to_string(), current_dir.to_string_lossy().into_owned());
        } else {
            eprintln!("Failed to get current working directory\r");
        }

        //ls and cat
        map.insert("ls".to_string(), "/home/youzar-boot/0-shell/bin/ls".to_string());
        map.insert("cat".to_string(), "/home/youzar-boot/0-shell/bin/cat".to_string());

        // $0 (program name / shell binary)
        let mut args = env::args();
        if let Some(shell_path) = args.next() {
            map.insert("$0".to_string(), shell_path);
        }

        for (i, arg) in args.enumerate() {
            let key = format!("${}", i + 1);
            map.insert(key, arg);
        }

        map
    });
}
