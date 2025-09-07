use crate::ShellCommand;
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
    fn execute(&self, env: &mut crate::envirement::ShellEnv) -> std::io::Result<()> {
        if self.args.len() > 1 {
            eprintln!("fg: too many arguments");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "fg: too many arguments",
            ));
        }

        // Determine which job to bring to foreground
        let job = if self.args.len() == 1 {
            // Try to parse job id from argument
            let arg = &self.args[0];
            match arg.parse::<usize>() {
                Ok(job_id) => match env.jobs.get_job(Pid::from_raw(job_id as i32)) {
                    Some(job) => job,
                    None => {
                        eprintln!("fg: no such job {}", job_id);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "fg: no such job",
                        ));
                    }
                },
                Err(_) => {
                    eprintln!("fg: invalid job id '{}'", arg);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "fg: invalid job id",
                    ));
                }
            }
        } else {
            // No argument: use last job
            match env.jobs.get_last_stopped_job() {
                Some(job) => job,
                None => {
                    eprintln!("fg: no stopped job");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "fg: no stopped job",
                    ));
                }
            }
        };
        let gid = job.pgid;
        // Send SIGCONT to the process group to continue it in the background
        if let Err(err) = nix::sys::signal::killpg(gid, Signal::SIGCONT) {
            eprintln!("bg: failed to send SIGCONT: {}", err);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "bg: failed to send SIGCONT",
            ));
        }
        env.jobs
            .update_job_status(job.pgid, crate::features::jobs::JobStatus::Running);
        println!("Job [{}] continued in background", gid);

        Ok(())
    }
}
