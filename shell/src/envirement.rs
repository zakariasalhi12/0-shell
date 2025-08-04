use std::collections::HashMap;
use std::time::SystemTime;

use crate::jobs::Job;
use crate::parser::types::AstNode;

use dirs::home_dir;
use lazy_static::lazy_static;
use std::fs::read_to_string;
use std::{env, sync::Mutex};
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
    /// Shell variables (like $PATH, $HOME)
    pub variables: HashMap<String, (String, bool)>,

    /// Arithmetic variables used in $((...)) and `let`-style expressions
    pub arith_vars: HashMap<String, i64>,

    /// User-defined shell functions
    pub functions: HashMap<String, AstNode>,

    /// Background/foreground jobs
    pub jobs: HashMap<usize, Job>,

    /// The next job ID (e.g. %1, %2, ...)
    pub next_job_id: usize,

    /// Whether the last command succeeded
    pub last_status: i32,

    /// Shell start time (can be used for uptime, etc.)
    pub started_at: SystemTime,
}

impl ShellEnv {
    /// Create a new default shell environment.

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
        variables.insert("~".to_string(), home);

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


        return Self {
            variables: std::env::vars().map(|k| (k.0, (k.1, true))).collect(),
            arith_vars: HashMap::new(),
            functions: HashMap::new(),
            jobs: HashMap::new(),
            next_job_id: 1,
            last_status: 0,
            started_at: SystemTime::now(),
        };

        // Example default vars if missing
        // env.variables
        //     .entry("PATH".to_string())
        //     .or_insert_with(|| "/usr/bin:/bin".to_string());
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
        if let Some(value) = self.variables.get(key) {
            Some(value.0.clone())
        } else {
            Some("".to_string())
        }
    }

    pub fn get_environment_only(&self) -> HashMap<String,String> {
        self.variables.iter().filter(|(_, v)| v.1).map(|(k, v)| (k.clone(), v.0.clone())).collect()
    }

    // pub fn get_env_

    /// Add a job and increment job ID
    pub fn add_job(&mut self, mut job: Job) -> usize {
        let id = self.next_job_id;
        self.next_job_id += 1;
        job.id = id;
        self.jobs.insert(id, job);
        id
    }

    /// Get job by ID
    pub fn get_job(&self, id: usize) -> Option<&Job> {
        self.jobs.get(&id)
    }

    /// Remove completed job
    pub fn remove_job(&mut self, id: usize) {
        self.jobs.remove(&id);
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
