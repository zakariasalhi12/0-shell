use crate::ShellCommand;
use crate::error::ShellError;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

pub struct Kill {
    args: Vec<String>,
}

enum ArgKind {
    Pid(i32),
    JobId(u32),
}

impl Kill {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
    pub fn validate_args(&self) -> Result<ArgKind, String> {
        if self.args.is_empty() {
            return Err("No args".to_string());
        } else if self.args.len() > 1 {
            return Err("Too much args".to_string());
        }

        let arg = &self.args[0];
        if let Some(stripped) = arg.strip_prefix('%') {
            match stripped.parse::<u32>() {
                Ok(job_id) => Ok(ArgKind::JobId(job_id)),
                Err(_) => Err("Invalid job ID".to_string()),
            }
        } else {
            match arg.parse::<i32>() {
                Ok(pid) => Ok(ArgKind::Pid(pid)),
                Err(_) => Err("Invalid PID".to_string()),
            }
        }
    }
}

impl ShellCommand for Kill {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> Result<i32, ShellError> {
        match self.validate_args() {
            Ok(ArgKind::Pid(pid_raw)) => {
                let pid = Pid::from_raw(pid_raw);
                match kill(pid, Signal::SIGKILL) {
                    Ok(_) => {
                        env.jobs
                            .update_job_status(pid, crate::features::jobs::JobStatus::Terminated);
                        // remove from both jobs map and order
                        env.jobs.remove_job(pid);
                        env.jobs.order.retain(|p| *p != pid);
                        env.jobs.update_job_marks();

                        Ok(0)
                    }
                    Err(e) => Err(ShellError::Exec(e.desc().to_owned())),
                }
            }
            Ok(ArgKind::JobId(job_id)) => {
                // find job by its id
                if let Some((pgid, job)) = env
                    .jobs
                    .jobs
                    .iter()
                    .find(|(_, j)| j.id == job_id)
                    .map(|(pgid, job)| (*pgid, job.clone()))
                {
                    match kill(job.pgid, Signal::SIGKILL) {
                        Ok(_) => {
                            env.jobs.update_job_status(
                                job.pid,
                                crate::features::jobs::JobStatus::Terminated,
                            );
                            env.jobs.remove_job(pgid);
                            env.jobs.order.retain(|p| *p != pgid);
                            env.jobs.update_job_marks();

                            Ok(0)
                        }
                        Err(e) => Err(ShellError::Exec(e.desc().to_owned())),
                    }
                } else {
                    Err(ShellError::Exec(format!("No such job: %{}", job_id)))
                }
            }
            Err(msg) => {
                if msg == "No args" {
                    if let Some(last_pid) = env.jobs.order.last().cloned() {
                        match kill(last_pid, Signal::SIGKILL) {
                            Ok(_) => {
                                env.jobs.update_job_status(
                                    last_pid,
                                    crate::features::jobs::JobStatus::Terminated,
                                );
                                env.jobs.remove_job(last_pid);
                                env.jobs.order.pop();
                                env.jobs.update_job_marks();

                                Ok(0)
                            }
                            Err(e) => Err(ShellError::Exec(e.desc().to_owned())),
                        }
                    } else {
                        Err(ShellError::Exec("No jobs to kill".to_string()))
                    }
                } else {
                    Err(ShellError::Exec(msg))
                }
            }
        }
    }
}
