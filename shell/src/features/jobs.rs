use std::{collections::HashMap, fmt::format};

use nix::unistd::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jobs {
    pub jobs: HashMap<Pid, Job>,
    pub size: u32,
    pub last_job: Option<Pid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub pgid: Pid,
    pub pids: Vec<Pid>,
    pub id: String,
    pub status: JobStatus,
    pub command: String,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: HashMap::new(),
            size: 0,
            last_job: None,
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.last_job = Some(job.pgid.clone());
        self.jobs.insert(job.pgid, job);
        self.size += 1;
    }

    pub fn get_last_job(&self) -> Option<&Job> {
        match self.last_job {
            Some(pid) => self.jobs.get(&pid),
            None => None,
        }
    }

    pub fn remove_job(&mut self, pid: Pid) {
        self.jobs.remove(&pid);
        self.size -= 1;
    }

    pub fn get_job(&self, pid: Pid) -> Option<&Job> {
        self.jobs.get(&pid)
    }
}

impl Job {
    pub fn new(pgid: Pid, pid: Pid, id: u32, status: JobStatus, command: String) -> Self {
        Job {
            pgid,
            pids: vec![pid],
            id : format!("%{}", id),
            status,
            command,
        }
    }

    pub fn add_pid(&mut self, pid: Pid) {
        self.pids.push(pid);
    }

    pub fn remove_pid(&mut self, pid: Pid) {
        self.pids.retain(|&p| p != pid);
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
    }

}