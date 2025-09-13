use crate::error::ShellError;
use crate::features::jobs::JobStatus;
use crate::{ShellCommand, envirement::ShellEnv};
use nix::unistd::Pid;

pub struct Jobs {
    pub args: Vec<String>,
}

impl Jobs {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    fn has_flag(&self, flag: &str) -> bool {
        self.args.iter().any(|a| a == flag)
    }
}

impl ShellCommand for Jobs {
    fn execute(&self, env: &mut ShellEnv) -> Result<i32, ShellError> {
        if env.jobs.jobs.is_empty() {
            return Ok(0);
        }

        // Use the order vector to maintain job creation order
        for (job_index, &pgid) in env.jobs.order.iter().enumerate() {
            if let Some(job) = env.jobs.jobs.get(&pgid) {
                // --- Apply filtering flags ---
                if self.has_flag("-r") && job.status != JobStatus::Running {
                    continue;
                }
                if self.has_flag("-s") && job.status != JobStatus::Stopped {
                    continue;
                }

                // Skip done/terminated jobs unless explicitly requested
                if matches!(job.status, JobStatus::Done | JobStatus::Terminated)
                    && !self.has_flag("-a")
                {
                    continue;
                }

                // --- Output mode ---
                if self.has_flag("-p") {
                    // PID-only mode - show the process group ID
                    println!("{}", job.pgid);
                } else {
                    // Normal / long format
                    let mut prev_or_next = String::new();
                    if job.prev_job {
                        prev_or_next = "-".to_owned();
                    }
                    if job.current_job {
                        prev_or_next = "+".to_owned();
                    }

                    let status_str = match job.status {
                        JobStatus::Running => "Running",
                        JobStatus::Stopped => "Stopped",
                        JobStatus::Terminated => "Terminated",
                        JobStatus::Done => "Done",
                    };

                    if self.has_flag("-l") {
                        // Long format: show detailed process information
                        if job.processes.is_empty() {
                            // Single command job
                            println!(
                                "[{}]{}  {}  {}    {}",
                                job.id, prev_or_next, job.pgid, status_str, job.command
                            );
                        } else {
                            // Pipeline job - show main job info first
                            println!(
                                "[{}]{}  {}  {}    {}",
                                job.id, prev_or_next, job.pgid, status_str, job.command
                            );

                            // Optionally show individual processes in pipeline
                            if self.has_flag("-v") || self.has_flag("--verbose") {
                                for (i, process) in job.processes.iter().enumerate() {
                                    let proc_status = match process.status {
                                        crate::features::jobs::ProcessStatus::Running => "Running",
                                        crate::features::jobs::ProcessStatus::Stopped => "Stopped",
                                        crate::features::jobs::ProcessStatus::Done => "Done",
                                        crate::features::jobs::ProcessStatus::Terminated => {
                                            "Terminated"
                                        }
                                    };
                                    println!(
                                        "     ├─ {} {} ({})",
                                        process.pid, process.command, proc_status
                                    );
                                }
                            }
                        }
                    } else {
                        // Default format
                        println!(
                            "[{}]{}{}{}    {}",
                            job.id, prev_or_next, " ".repeat(2 - prev_or_next.len()), status_str, job.command
                        );
                    }
                }
            }
        }

        Ok(0)
    }
}
