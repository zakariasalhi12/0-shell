use lazy_static::lazy_static;
use std::{collections::HashMap, env, sync::Mutex};
use dirs::home_dir;
use whoami;
use std::fs::read_to_string;


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
    let username = whoami::username();
    map.insert("USER".to_string(), username.clone());
        
    map.insert("HOME".to_string(), 
        home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .or_else(|| env::var("HOME").ok())
            .unwrap_or_else(|| "/".to_string())
    );

    let shell = get_user_shell(username.as_str()).expect("not found");
    map.insert(String::from("SHELL"), shell);
    
    // map.insert("SHELL", get_user_shell())



        map
    });
}