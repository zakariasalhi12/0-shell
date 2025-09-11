use std::collections::HashMap;

use nix::unistd::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Terminated,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Terminated,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub status: ProcessStatus,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: Pid, command: String) -> Self {
        Self {
            pid,
            status: ProcessStatus::Running,
            command,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, ProcessStatus::Done | ProcessStatus::Terminated)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self.status, ProcessStatus::Stopped)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, ProcessStatus::Running)
    }
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
    pub pid: Pid,                    // Leader PID (same as pgid for pipelines)
    pub processes: Vec<ProcessInfo>, // All processes in the job/pipeline
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

    pub fn get_prev_job(&self) -> Option<&Job> {
        match self.prev_job {
            Some(pid) => self.jobs.get(&pid),
            None => None,
        }
    }

    pub fn remove_job(&mut self, pgid: Pid) {
        self.jobs.remove(&pgid);
        self.order.retain(|&p| p != pgid);
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

    // Find job by any PID in the job (including processes in pipeline)
    pub fn find_job_by_any_pid(&mut self, pid: Pid) -> Option<&mut Job> {
        self.jobs
            .values_mut()
            .find(|job| job.pid == pid || job.processes.iter().any(|p| p.pid == pid))
    }

    // Update status of a specific process within a job
    pub fn update_process_status(&mut self, pid: Pid, status: ProcessStatus) -> bool {
        if let Some(job) = self.find_job_by_any_pid(pid) {
            // Update the specific process
            if let Some(process) = job.processes.iter_mut().find(|p| p.pid == pid) {
                process.status = status;

                // Update overall job status based on all processes
                job.update_overall_status();
                job.status.printStatus(job.clone());

                return job.all_processes_finished();
            }
        }
        false
    }

    // Legacy method for backward compatibility
    pub fn update_job_status(&mut self, pid: nix::unistd::Pid, status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&pid) {
            job.status = status;
            job.status.printStatus(job.clone());
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
            pid,
            processes: Vec::new(),
            id,
            status,
            command,
            current_job: false,
            prev_job: false,
        }
    }

    // Add a process to this job (for pipelines)
    pub fn add_process(&mut self, pid: Pid, command: String) {
        self.processes.push(ProcessInfo::new(pid, command));
    }

    // Remove a process from this job
    pub fn remove_process(&mut self, pid: Pid) {
        self.processes.retain(|p| p.pid != pid);
    }

    // Check if all processes in the job have finished
    pub fn all_processes_finished(&self) -> bool {
        !self.processes.is_empty() && self.processes.iter().all(|p| p.is_finished())
    }

    // Check if any process in the job is running
    pub fn any_process_running(&self) -> bool {
        self.processes.iter().any(|p| p.is_running())
    }

    // Check if any process in the job is stopped
    pub fn any_process_stopped(&self) -> bool {
        self.processes.iter().any(|p| p.is_stopped())
    }

    // Update overall job status based on individual process statuses
    pub fn update_overall_status(&mut self) {
        if self.processes.is_empty() {
            return;
        }

        if self.all_processes_finished() {
            // If all processes are done, check if any were terminated
            if self
                .processes
                .iter()
                .any(|p| matches!(p.status, ProcessStatus::Terminated))
            {
                self.status = JobStatus::Terminated;
            } else {
                self.status = JobStatus::Done;
            }
        } else if self.any_process_stopped() {
            self.status = JobStatus::Stopped;
        } else if self.any_process_running() {
            self.status = JobStatus::Running;
        }
    }

    // Legacy method for backward compatibility
    pub fn add_pid(&mut self, pid: Pid) {
        self.add_process(pid, "unknown".to_string());
    }

    // Legacy method for backward compatibility
    pub fn remove_pid(&mut self, pid: Pid) {
        self.remove_process(pid);
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
    }
}
