use std::collections::HashMap;

use nix::unistd::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Terminated,
    Done,
}
//[id][prev or current] [status]  [command]
// [1] 8287
impl JobStatus {
    pub fn printStatus(&self, job: Job) {
        let mut prev_or_next = String::new();

        if job.prev_job {
            prev_or_next = "-".to_owned()
        }
        if job.current_job {
            prev_or_next = "+".to_owned()
        }

        match self {
            Self::Running => {
                println!("[{}]{} {}\r", job.id, prev_or_next, job.pgid);
            }
            Self::Done => {
                // println!("\n[{}]{}  Done {} {}\r" , job.id , prev_or_next , " ".repeat(5) , job.command);
            }
            Self::Stopped => {
                println!(
                    "[{}]{}  Stopped {} {}\r",
                    job.id,
                    prev_or_next,
                    " ".repeat(5),
                    job.command
                );
            }
            Self::Terminated => {
                // println!("\n[{}]{}  Terminated {} {}\r" , job.id , prev_or_next , " ".repeat(5) , job.command);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jobs {
    pub jobs: HashMap<Pid, Job>,
    pub size: u32,
    pub current_job: Option<Pid>,
    pub prev_job: Option<Pid>,
    pub order: Vec<Pid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub pgid: Pid,
    pub pid: Pid,
    pub pids: Vec<Pid>,
    pub id: u32,
    pub status: JobStatus,
    pub command: String,
    pub prev_job: bool,
    pub current_job: bool,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: HashMap::new(),
            size: 0,
            prev_job: None,
            current_job: None,
            order: vec![],
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.current_job = Some(job.pgid);
        self.order.push(job.pgid.clone());
        self.jobs.insert(job.pgid, job.clone());
        self.size += 1;
        self.update_job_marks();
    }

    pub fn get_current_job(&self) -> Option<&Job> {
        match self.current_job {
            Some(pid) => self.jobs.get(&pid),
            None => None,
        }
    }

   fn update_job_marks(&mut self) {
    self.current_job = None;
    self.prev_job = None;
    // Clear all job marks
    for job in self.jobs.values_mut() {
        job.current_job = false;
        job.prev_job = false;
    }
    // Find jobs in reverse order (most recent first)
    let mut stopped = Vec::new();
    let mut running = Vec::new();
    for &pid in self.order.iter().rev() {
        if let Some(job) = self.jobs.get(&pid) {
            match job.status {
                JobStatus::Stopped => stopped.push(pid),
                JobStatus::Running => running.push(pid),
                _ => {}
            }
        }
    }
    let mut candidates = stopped;
    if candidates.is_empty() {
        candidates = running;
    }
    if let Some(&cur) = candidates.get(0) {
        self.current_job = Some(cur);
        if let Some(job) = self.jobs.get_mut(&cur) {
            job.current_job = true;
        }
    }
    if let Some(&prev) = candidates.get(1) {
        self.prev_job = Some(prev);
        if let Some(job) = self.jobs.get_mut(&prev) {
            job.prev_job = true;
        }
    }
}

    fn set_prev(&mut self, job: Job) {}

    pub fn get_prev_job(&self) -> Option<&Job> {
        match self.prev_job {
            Some(pid) => self.jobs.get(&pid),
            None => None,
        }
    }

    pub fn remove_job(&mut self, pid: Pid) {
        self.jobs.remove(&pid);
        self.order.retain(|&p| p != pid);
        if self.size > 0 {
            self.size -= 1;
        }
        self.update_job_marks();
    }

    pub fn get_job(&self, pid: Pid) -> Option<&Job> {
        self.jobs.get(&pid)
    }

    pub fn get_job_mut(&mut self, pid: Pid) -> Option<&mut Job> {
        self.jobs.get_mut(&pid)
    }

    pub fn get_job_byid(&self, id: u32) -> Option<&Job> {
        self.jobs.values().find(|job| job.id == id)
    }

    pub fn update_job_status(&mut self, pid: nix::unistd::Pid, status: JobStatus) {
        if let Some(job) = self.jobs.iter_mut().find(|job| *job.0 == pid) {
            job.1.status = status;
            job.1.status.printStatus(job.1.to_owned());
        }
    }

    pub fn get_last_stopped_job(&self) -> Option<&Job> {
        for pid in self.order.iter().rev() {
            if let Some(job) = self.jobs.get(pid) {
                if job.status == JobStatus::Stopped {
                    return Some(job);
                }
            }
        }
        None
    }
}

impl Job {
    pub fn new(pgid: Pid, pid: Pid, id: u32, status: JobStatus, command: String) -> Self {
        Job {
            pgid,
            pid: pid,
            pids: vec![pid],
            id: id,
            status,
            command,
            current_job: false,
            prev_job: false,
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
