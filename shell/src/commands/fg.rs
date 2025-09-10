use std::io::Write;
use crate::error::ShellError;
use crate::{
    ShellCommand,
    features::jobs::JobStatus,
};
use nix::sys::signal::{Signal, signal};

pub struct Fg {
    args: Vec<String>,
}

impl Fg {
    pub fn new(args: Vec<String>) -> Self {
        Fg { args }
    }
}

impl ShellCommand for Fg {
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> Result<i32, ShellError> {
        if self.args.len() == 0 && env.jobs.size == 0 {
            return Err(ShellError::Exec(String::from(format!("fg: no current job"))));
        }

        let job = if self.args.len() >= 1 {
            let first_arg = &self.args[0];
            if !first_arg.starts_with('%') {
                return Err(ShellError::Exec(format!("fg: job not found: {}", first_arg)));
            }

            let id = match first_arg[1..].parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    return Err(ShellError::Exec(format!("fg: job not found: {}", first_arg)));
                }
            };

            env.jobs
                .get_job_byid(id)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("fg: job {} not found", id),
                    )
                })?
                .to_owned()
        } else {
            env.jobs
                .get_current_job()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "fg: no current job")
                })?
                .to_owned()
        };
        let pgid = job.pgid;

        // Ignore SIGTTOU so shell doesn't get suspended
        let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set SIGTTOU signal handler: {}", e)
            ))?;

        // Move job to foreground
        if let Err(e) = nix::unistd::tcsetpgrp(nix::libc::STDIN_FILENO, pgid) {
            // Restore signal handler before returning error
            unsafe { 
                let _ = signal(Signal::SIGTTOU, old);
            }
            return Err(ShellError::Exec(String::from(format!("Failed to set terminal control: {}", e))));
        }

        // Restore SIGTTOU
        unsafe { 
            signal(Signal::SIGTTOU, old).map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to restore SIGTTOU signal handler: {}", e)
            ))?;
        }

        // Resume the job if it was stopped
        if let Err(e) = nix::sys::signal::killpg(pgid, nix::sys::signal::SIGCONT) {
            return Err(ShellError::Exec(String::from(format!("Failed to send SIGCONT to process group: {}", e))));
        }

        // Wait for the job to finish or stop again
        match nix::sys::wait::waitpid(pgid, Some(nix::sys::wait::WaitPidFlag::WUNTRACED)) {
            Ok(wait_status) => match wait_status {
                nix::sys::wait::WaitStatus::Exited(_, _) => {
                    // Remove job when process exits
                    env.jobs.remove_job(pgid);
                }
                nix::sys::wait::WaitStatus::Signaled(_, _, _) => {
                    env.jobs.remove_job(pgid);
                    if let Err(e) = std::io::stdout().flush() {
                        eprintln!("Warning: Failed to flush stdout: {}", e);
                    }
                }
                nix::sys::wait::WaitStatus::Stopped(_, _) => {
                    println!("[{}]+ Stopped", pgid);
                    env.jobs.update_job_status(pgid, JobStatus::Stopped);
                }
                other => {
                    return Err(ShellError::Exec(format!("Unexpected wait status: {:?}", other)));
                }
            },
            Err(e) => {
                return Err(ShellError::Exec(format!("waitpid failed: {}", e)));
            }
        }

        // Return terminal control to shell
        let shell_pgid = nix::unistd::getpgrp();
        let old = unsafe { 
            signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn).map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set SIGTTOU signal handler: {}", e)
            ))?
        };

        if let Err(e) = nix::unistd::tcsetpgrp(nix::libc::STDIN_FILENO, shell_pgid) {
            return Err(ShellError::Exec(String::from(format!("Failed to return terminal control to shell: {}", e))));
        }

        unsafe {
            signal(Signal::SIGTTOU, old).map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to restore SIGTTOU signal handler: {}", e)
            ))?
        };

        Ok(0)
    }
}
