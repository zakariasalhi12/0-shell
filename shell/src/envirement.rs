use std::collections::HashMap;
use std::time::SystemTime;

use crate::features::jobs::Jobs;
use crate::parser::types::AstNode;

use dirs::home_dir;
use std::env;
use std::fs::read_to_string;
use whoami;

fn get_user_shell(username: &str) -> Option<String> {
    read_to_string("/etc/passwd")
        .ok()?
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(6).map(String::from))
}

/// Represents the current shell environment.
#[derive(Clone)]

pub struct ShellEnv {
    pub variables: HashMap<String, (String, bool)>,
    pub arith_vars: HashMap<String, i64>,
    pub functions: HashMap<String, AstNode>,
    pub jobs: Jobs,
    pub next_job_id: usize,
    pub last_status: i32,
    pub started_at: SystemTime,
    pub current_command: String,
}

impl ShellEnv {
    pub fn new() -> Self {
        let mut variables: HashMap<String, (String, bool)> = HashMap::new();

        // inherited env vars
        for env_var in std::env::vars() {
            variables.insert(env_var.0, (env_var.1, true));
        }

        let username = whoami::username();
        variables.insert("USER".to_string(), (username.clone(), true));

        // HOME and ~
        let home = home_dir()
            .map(|p| (p.to_string_lossy().into_owned(), true))
            .or_else(|| env::var("HOME").ok().map(|p| (p, true)))
            .unwrap_or_else(|| ("/".to_string(), true));

        variables.insert("HOME".to_string(), home.clone());
        variables.insert("~".to_string(), home.clone());
        // SHELL
        let shell = get_user_shell(&username).unwrap_or_default();
        variables.insert("SHELL".to_string(), (shell, true));

        // PWD (current directory)
        if let Ok(current_dir) = env::current_dir() {
            variables.insert(
                "PWD".to_string(),
                (current_dir.to_string_lossy().into_owned(), true),
            );
        } else {
            eprintln!("Failed to get current working directory\r");
        }

        // positional arguments
        let args = env::args();
        for (i, arg) in args.enumerate() {
            let key: String = format!("{}", i);
            variables.insert(key, (arg, true));
        }

        variables.extend(std::env::vars().map(|k| (k.0, (k.1, true))));

        return Self {
            variables,
            arith_vars: HashMap::new(),
            functions: HashMap::new(),
            jobs: Jobs::new(),
            next_job_id: 1,
            last_status: 0,
            started_at: SystemTime::now(),
            current_command: String::new(),
        };
    }

    /// Set a shell variable
    pub fn set_local_var(&mut self, key: &str, value: &str) {
        self.variables
            .insert(key.to_string(), (value.to_string(), false));
    }

    pub fn set_env_var(&mut self, key: &str, value: &str) {
        self.variables
            .insert(key.to_string(), (value.to_string(), true));
    }

    /// Get a shell variable
    pub fn get(&self, key: &str) -> Option<String> {
        if key == "?" {
            return Some(self.last_status.to_string());
        } else if let Some(value) = self.variables.get(key) {
            Some(value.0.clone())
        } else {
            Some("".to_string())
        }
    }

    pub fn get_environment_only(&self) -> HashMap<String, String> {
        self.variables
            .iter()
            .filter(|(_, v)| v.1)
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect()
    }

    pub fn last_job_pid(&self) -> Option<i32> {
        self.jobs.get_last_job().map(|job| job.pgid.as_raw())
    }

    /// Set last command exit status ($?)
    pub fn set_last_status(&mut self, status: i32) {
        self.last_status = status;
    }

    /// Get last command exit status
    pub fn get_last_status(&self) -> i32 {
        self.last_status
    }

    /// Set Function into functions var
    pub fn set_func(&mut self, name: String, function: AstNode) {
        self.functions.insert(name, function);
    }

    /// get Function into functions var
    pub fn get_func(&mut self, name: &str) -> Option<&AstNode> {
        self.functions.get(name)
    }
}
