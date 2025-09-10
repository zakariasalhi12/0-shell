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
            let status_str = match job.status {
                JobStatus::Running => "Running",
                JobStatus::Stopped => "Stopped",
                JobStatus::Terminated => "Terminated",
                JobStatus::Done => unreachable!(),
            };

            // Example output: [1]  + 12345 running    sleep 10
            println!("[{}]  {}  {}    {}", i, id, status_str, job.command);
            i += 1;
        }
        Ok(0)
    }
}
