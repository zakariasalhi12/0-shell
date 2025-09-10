use crate::error::ShellError;
use crate::ShellCommand;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

pub struct Kill {
    args: Vec<String>,
}

impl Kill {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    pub fn validate_args(&self) -> Result<i32, String> {
        if self.args.len() == 0 {
            return Err("No args".to_string());
        } else if self.args.len() > 1 {
            return Err("Too much args".to_string());
        } else {
            match self.args[0].parse::<i32>() {
                Ok(pid) => Ok(pid),
                Err(_) => Err("Invalid PID".to_string()),
            }
        }
    }
}

impl ShellCommand for Kill {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> Result<i32, ShellError> {
        match self.validate_args() {
            Ok(pid) => match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(_) => {
                    env.jobs.remove_job(Pid::from_raw(pid)); // Clean up job after killing
                    Ok(0)
                }
                Err(e) => Err(ShellError::Exec(e.desc().to_owned()))},
            Err(msg) => {
                if msg == "No args" {
                    // Try to get the last job from env and kill it
                    if let Some(last_job_pid) = env.last_job_pid() {
                        match kill(Pid::from_raw(last_job_pid), Signal::SIGKILL) {
                            Ok(_) => {
                                env.jobs.remove_job(Pid::from_raw(last_job_pid)); // Clean up job after killing
                                Ok(0)
                            }
                            Err(e) => Err(ShellError::Exec(e.desc().to_owned()))
,
                        }
                    } else {
                        Err(ShellError::Exec("No jobs to kill".to_string()))
                    }
                } else {
                    Err(ShellError::Exec(msg.to_owned()))
                }
            }
        }
    }
}
