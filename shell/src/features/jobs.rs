use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jobs {
    pub jobs: HashMap<u32, Job>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub pgid: u32,
    pub pid: u32,
    pub id: String,
    pub status: JobStatus,
    pub command: String,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.jobs.insert(job.pid, job);
    }

    pub fn remove_job(&mut self, pid: u32) {
        self.jobs.remove(&pid);
    }

    pub fn get_job(&self, pid: u32) -> Option<&Job> {
        self.jobs.get(&pid)
    }
}

impl Job {
    pub fn new(pid: u32, id: String, status: JobStatus, command: String) -> Self {
        Job {
            pid,
            id,
            status,
            command,
        }
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
    }

    pub fn display(&self) -> String {
        format!("Job ID: {}, PID: {}, Status: {:?}, Command: {}", self.id, self.pid, self.status, self.command)
    }
}