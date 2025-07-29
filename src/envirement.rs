use std::collections::HashMap;
use std::time::SystemTime;

use crate::jobs::Job;
use crate::parser::types::AstNode;

/// Represents the current shell environment.
pub struct ShellEnv {
    /// Shell variables (like $PATH, $HOME)
    pub variables: HashMap<String, String>,

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
        let mut env = ShellEnv {
            variables: std::env::vars().collect(),
            arith_vars: HashMap::new(),
            functions: HashMap::new(),
            jobs: HashMap::new(),
            next_job_id: 1,
            last_status: 0,
            started_at: SystemTime::now(),
        };

        // Example default vars if missing
        env.variables
            .entry("PATH".to_string())
            .or_insert_with(|| "/usr/bin:/bin".to_string());
        env
    }

    /// Set a shell variable
    pub fn set_var(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Get a shell variable
    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Set an arithmetic variable
    pub fn set_arith(&mut self, key: &str, value: i64) {
        self.arith_vars.insert(key.to_string(), value);
    }

    /// Get an arithmetic variable
    pub fn get_arith(&self, key: &str) -> Option<i64> {
        self.arith_vars.get(key).cloned()
    }

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
}
