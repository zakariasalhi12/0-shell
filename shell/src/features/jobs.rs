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
    pub pids: Vec<u32>,
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
        self.jobs.insert(job.pgid, job);
    }

    pub fn remove_job(&mut self, pid: u32) {
        self.jobs.remove(&pid);
    }

    pub fn get_job(&self, pid: u32) -> Option<&Job> {
        self.jobs.get(&pid)
    }
}

impl Job {
    pub fn new(pgid: u32, pid: u32, id: String, status: JobStatus, command: String) -> Self {
        Job {
            pgid,
            pids: vec![pid],
            id,
            status,
            command,
        }
    }

    pub fn add_pid(&mut self, pid: u32) {
        self.pids.push(pid);
    }

    pub fn remove_pid(&mut self, pid: u32) {
        self.pids.retain(|&p| p != pid);
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
    }

}