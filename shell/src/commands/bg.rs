use crate::features::jobs::JobStatus;
use crate::{ShellCommand, error::ShellError};
use nix::{
    sys::signal::{Signal, signal},
    unistd::Pid,
};
pub struct Bg {
    args: Vec<String>,
}

impl Bg {
    pub fn new(args: Vec<String>) -> Self {
        Bg { args }
    }
}

impl ShellCommand for Bg {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> Result<i32, ShellError> {
        if self.args.len() > 1 {
            return Err(ShellError::Exec(String::from("fg: too many arguments")));
        }

        // Determine which job to bring to foreground
        let job = if self.args.len() == 1 {
            // Try to parse job id from argument
            let arg = &self.args[0];
            match arg.parse::<usize>() {
                Ok(job_id) => match env.jobs.get_job(Pid::from_raw(job_id as i32)) {
                    Some(job) => job,
                    None => {
                        return Err(ShellError::Exec(format!("fg: no such job {}", job_id)));
                    }
                },
                Err(_) => {
                    return Err(ShellError::Exec(format!("fg: invalid job id '{}'", arg)));
                }
            }
        } else {
            // No argument: use last job
            match env.jobs.get_last_stopped_job() {
                Some(job) => job,
                None => {
                    return Err(ShellError::Exec(String::from("fg: no stopped job")));
                }
            }
        };
        let gid = job.pgid;
        // Send SIGCONT to the process group to continue it in the background
        if let Err(err) = nix::sys::signal::killpg(gid, Signal::SIGCONT) {
            return Err(ShellError::Exec(String::from("bg: failed to send SIGCONT")));
        }
        env.jobs
            .update_job_status(job.pgid, crate::features::jobs::JobStatus::Running);

        Ok(0)
    }
}
