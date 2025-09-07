use crate::features::jobs::JobStatus;
use crate::{ShellCommand, envirement::ShellEnv};
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
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        if self.args.len() > 1 {
            eprintln!("fg: too many arguments");
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("idk"),
            ));
        }

        let job = env.jobs.get_last_job().unwrap();
        let pgid = job.pgid;

        // Ignore SIGTTOU so shell doesn't get suspended
        let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }.unwrap();

        // Move job to foreground
        if let Err(e) = nix::unistd::tcsetpgrp(nix::libc::STDIN_FILENO, pgid) {
            eprintln!("fg: failed to set terminal control: {}", e);
            unsafe { signal(Signal::SIGTTOU, old).unwrap() };
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "tcsetpgrp failed",
            ));
        }

        // Restore SIGTTOU
        unsafe { signal(Signal::SIGTTOU, old).unwrap() };

        // Resume the job if it was stopped
        nix::sys::signal::killpg(pgid, nix::sys::signal::SIGCONT).ok();

        // Wait for the job to finish or stop again
        loop {
            match nix::sys::wait::waitpid(pgid, Some(nix::sys::wait::WaitPidFlag::WUNTRACED)) {
                Ok(wait_status) => match wait_status {
                    nix::sys::wait::WaitStatus::Exited(_, code) => {
                        // Remove job when process exits
                        env.jobs.remove_job(pgid);
                    }
                    nix::sys::wait::WaitStatus::Signaled(_, _, _) => {
                        env.jobs.remove_job(pgid);
                    }
                    nix::sys::wait::WaitStatus::Stopped(_, _) => {
                        println!("[{}]+ Stopped", pgid);
                        env.jobs.update_job_status(pgid, JobStatus::Stopped);
                    }
                    _ => break,
                },
                Err(_) => break,
                // No need for a wildcard arm anymore
            }
        }

        // Return terminal control to shell
        let shell_pgid = nix::unistd::getpgrp();
        let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }.unwrap();
        nix::unistd::tcsetpgrp(nix::libc::STDIN_FILENO, shell_pgid).ok();
        unsafe { signal(Signal::SIGTTOU, old).unwrap() };

        Ok(())
    }
}
