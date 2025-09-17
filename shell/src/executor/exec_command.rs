use crate::{
    error::ShellError,
    exec::{CommandResult, wait_for_single_process},
    executor::Executor,
    executorr::spawn_commande::spawn_command,
    features::jobs::{Job, JobStatus},
    lexer::types::{QuoteType, Word},
    types::AstNode,
};

impl<'a> Executor<'a> {
    pub fn exec_command(&mut self, node: &AstNode, is_background: bool) -> Result<i32, ShellError> {
        if let AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } = node
        {
            match spawn_command(cmd, args, assignments, redirects, self.env, None, &mut None)? {
                CommandResult::Child(pid) => {
                    let merged = Word {
                        parts: args.iter().flat_map(|w| w.parts.clone()).collect(),
                        quote: QuoteType::None, // or however you want to handle quotes
                    };
                    if !is_background {
                        let status = wait_for_single_process(
                            pid,
                            self.env,
                            cmd.expand(self.env) + " " + &merged.expand(self.env),
                        )?;
                        self.env.set_last_status(status);
                        return Ok(status);
                    } else {
                        // Add to jobs and don't wait

                        let new_job = Job::new(
                            pid,
                            pid,
                            self.env.jobs.size + 1,
                            JobStatus::Running,
                            cmd.expand(self.env) + " " + &merged.expand(self.env),
                        );
                        self.env.jobs.add_job(new_job.clone());
                        new_job.status.printStatus(new_job.clone());
                        return Ok(0);
                    }
                }
                CommandResult::Builtin(n) => return Ok(n),
            }
        }
        unreachable!()
    }
}
