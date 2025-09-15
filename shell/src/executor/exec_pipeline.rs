use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

use nix::unistd::Pid;
use nix::unistd::pipe;
use nix::unistd::setpgid;

use crate::exec::wait_for_pipeline;
use crate::{
    error::ShellError,
    exec::CommandResult,
    executor::Executor,
    executorr::spawn_commande::spawn_command,
    features::jobs::{Job, JobStatus},
    types::AstNode,
};

impl<'a> Executor<'a> {
    pub fn exec_pipeline(
        &mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        if let AstNode::Pipeline(nodes) = node {
            if nodes.is_empty() {
                return Ok(0);
            }

            if nodes.len() == 1 {
                return self.execute_node(&nodes[0], is_background, loop_depth);
            }

            let mut prev_read: Option<OwnedFd> = None;
            let mut pipeline_gid = Option::<Pid>::None; // This will store the pipeline's process group ID
            let mut child_pids = Vec::<Pid>::new();

            // Execute all commands in the pipeline concurrently
            for (i, node) in nodes.iter().enumerate() {
                let is_last = i == nodes.len() - 1;
                let is_first = i == 0;

                if let AstNode::Command {
                    cmd,
                    args,
                    assignments,
                    redirects,
                } = node
                {
                    let stdin = prev_read.take();

                    // Create new pipe only if this is not the last command
                    let (read_end, write_end) = if !is_last {
                        let (read_fd, write_fd) = pipe().expect("pipe failed");
                        (
                            Some(unsafe { OwnedFd::from_raw_fd(read_fd.as_raw_fd()) }),
                            Some(unsafe { OwnedFd::from_raw_fd(write_fd.as_raw_fd()) }),
                        )
                    } else {
                        (None, None)
                    };

                    let fds_map = {
                        let mut map: HashMap<u64, OwnedFd> = HashMap::new();
                        if let Some(stdi) = stdin {
                            map.insert(0, stdi);
                        }
                        if let Some(stdo) = write_end {
                            map.insert(1, stdo);
                        }
                        Some(map)
                    };

                    // For the first command, we need to create a new process group
                    // For subsequent commands, we add them to the existing group
                    let mut current_gid = if is_first {
                        None // Let spawn_command create a new group
                    } else {
                        pipeline_gid // Use the established group
                    };

                    // Spawn the command without waiting
                    match spawn_command(
                        cmd,
                        args,
                        assignments,
                        redirects,
                        self.env,
                        fds_map.as_ref(),
                        &mut current_gid,
                    )? {
                        CommandResult::Child(child_pid) => {
                            child_pids.push(child_pid);

                            // If this is the first command, its PID becomes the process group ID
                            if is_first {
                                pipeline_gid = Some(child_pid);

                                // Make sure the first process becomes the group leader
                                // This should be done in spawn_command, but ensure it here
                                if let Err(e) = setpgid(child_pid, child_pid) {
                                    eprintln!("Warning: Failed to set process group leader: {}", e);
                                }
                            } else {
                                // Add subsequent processes to the pipeline's process group
                                if let Some(pgid) = pipeline_gid {
                                    if let Err(e) = setpgid(child_pid, pgid) {
                                        eprintln!("Warning: Failed to add process to group: {}", e);
                                    }
                                }
                            }
                        }
                        CommandResult::Builtin(n) => {
                            // Builtin commands are executed immediately
                            // In a real pipeline, builtins should also fork, but this is a simplification
                        }
                    }

                    prev_read = read_end; // becomes stdin for next command
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }

            // Now handle waiting based on whether it's background or not
            if !child_pids.is_empty() {
                if let Some(pgid) = pipeline_gid {
                    let pipeline_cmd = nodes
                        .iter()
                        .filter_map(|node| {
                            if let AstNode::Command { cmd, .. } = node {
                                Some(cmd.expand(self.env))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" | ");

                    if is_background {
                        // Create job and add all processes to it
                        let mut new_job = Job::new(
                            pgid,
                            pgid, // leader_pid is same as pgid for pipelines
                            self.env.jobs.size + 1,
                            JobStatus::Running,
                            pipeline_cmd,
                        );

                        // Add all child processes to the job
                        for (i, &pid) in child_pids.iter().enumerate() {
                            let cmd_name = if let AstNode::Command { cmd, .. } = &nodes[i] {
                                cmd.expand(self.env)
                            } else {
                                "unknown".to_string()
                            };
                            new_job.add_process(pid, cmd_name);
                        }

                        self.env.jobs.add_job(new_job);
                        return Ok(0);
                    } else {
                        // For foreground pipelines, we don't add to jobs but still wait properly
                        let status = wait_for_pipeline(pgid, child_pids, pipeline_cmd, self.env)?;
                        self.env.set_last_status(status);
                        return Ok(status);
                    }
                }
            }

            Ok(0)
        } else {
            Ok(0)
        }
    }
}
