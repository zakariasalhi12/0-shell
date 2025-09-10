use crate::error::ShellError;
use crate::features::jobs::JobStatus;
use crate::{ShellCommand, envirement::ShellEnv};

pub struct Jobs {
    pub args: Vec<String>,
}

impl Jobs {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl ShellCommand for Jobs {
    fn execute(&self, env: &mut ShellEnv) -> Result<i32, ShellError> {
        let mut i = 1;
        for (id, job) in &env.jobs.jobs {
            let mut prev_or_next = String::new();

            if job.prev_job {
                prev_or_next = "-".to_owned()
            }
            if job.current_job {
                prev_or_next = "+".to_owned()
            }

            let status_str = match job.status {
                JobStatus::Running => "Running",
                JobStatus::Stopped => "Stopped",
                JobStatus::Terminated => "Terminated",
                JobStatus::Done => "Done",
            };

            // Example output: [1]  + 12345 running    sleep 10
            println!("[{}]{}  {}  {}    {}", i, prev_or_next, id, status_str, job.command);
            i += 1;
        }
        Ok(0)
    }
}
