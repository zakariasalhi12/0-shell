use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::CommandResult;
use crate::exec::CommandType;
use crate::exec::execute;
use crate::exec::get_command_type;
use crate::executor::run_commande::run_commande;
use crate::expansion::expand_and_split;
use crate::features::jobs;
use crate::features::jobs::JobStatus;
use crate::lexer::types::Word;
use crate::redirection::setup_redirections_ownedfds;
use crate::types::Redirect;
use nix::sys::signal::Signal;
use nix::sys::signal::signal;
use nix::unistd::Pid;
use nix::unistd::getpgrp;
use nix::unistd::tcsetpgrp;
use std::collections::HashMap;
use std::fs::File;
use std::os::fd::IntoRawFd;
use std::os::unix::io::OwnedFd;
use std::vec;

pub fn invoke_command(
    cmd: &Word,
    args: &Vec<Word>,
    assignments: &Vec<(String, Word)>,
    redirects: &Vec<Redirect>,
    env: &mut ShellEnv,
    piping_fds: Option<&HashMap<u64, OwnedFd>>,
    gid: &mut Option<Pid>,
    is_background: bool,
) -> Result<i32, ShellError> {
    match spawn_command(cmd, args, assignments, redirects, env, piping_fds, gid)? {
        CommandResult::Child(pid) => {
            if !is_background {
                // This should now be handled in exec.rs
                // For compatibility, we'll wait here, but ideally exec.rs should handle this

                let tty = File::open("/dev/tty").unwrap();
                let fd = tty.into_raw_fd();

                let new_job = jobs::Job::new(
                    gid.unwrap_or(pid),
                    pid,
                    env.jobs.size + 1,
                    jobs::JobStatus::Running,
                    cmd.expand(env),
                );
                env.jobs.add_job(new_job);

                let shell_pgid = getpgrp();
                let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
                    .unwrap();
                tcsetpgrp(fd, pid).unwrap();
                unsafe { signal(Signal::SIGTTOU, old).unwrap() };

                let exitcode = match nix::sys::wait::waitpid(
                    pid,
                    Some(nix::sys::wait::WaitPidFlag::WUNTRACED),
                ) {
                    Ok(wait_status) => match wait_status {
                        nix::sys::wait::WaitStatus::Exited(_, code) => {
                            env.jobs.remove_job(pid);
                            code
                        }
                        nix::sys::wait::WaitStatus::Signaled(_, _, _) => {
                            env.jobs.remove_job(pid);
                            1
                        }
                        nix::sys::wait::WaitStatus::Stopped(_, _) => {
                            env.jobs.update_job_status(pid, JobStatus::Stopped);
                            println!();
                            1
                        }
                        _ => 1,
                    },
                    Err(_) => 1,
                };

                let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
                    .unwrap();
                tcsetpgrp(fd, shell_pgid).ok();
                unsafe { signal(Signal::SIGTTOU, old).unwrap() };

                env.set_last_status(exitcode);
                Ok(exitcode)
            } else {
                Ok(0)
            }
        }
        CommandResult::Builtin => Ok(0),
    }
}

pub fn spawn_command(
    cmd: &Word,
    args: &Vec<Word>,
    assignments: &Vec<(String, Word)>,
    redirects: &Vec<Redirect>,
    env: &mut ShellEnv,
    piping_fds: Option<&HashMap<u64, OwnedFd>>,
    gid: &mut Option<Pid>,
) -> Result<CommandResult, ShellError> {
    // 1. Expand command and args
    let mut all_args: Vec<String> = vec![];
    let mut expanded_command = expand_and_split(cmd, env);
    let cmd_str = if expanded_command.len() >= 1 {
        expanded_command.remove(0)
    } else {
        "".to_string()
    };
    all_args.extend(expanded_command);

    for arg in args {
        let expanded_args = expand_and_split(arg, env);
        all_args.extend(expanded_args);
    }

    // 2. Merge piping FDs and redirection FDs
    let merged_fds: Option<HashMap<u64, OwnedFd>> = match (!redirects.is_empty(), piping_fds) {
        // Case 1: We have redirects but no piping FDs
        (true, None) => Some(setup_redirections_ownedfds(&redirects, env)?),
        // Case 2: We have both redirects and piping FDs - need to merge
        (true, Some(piping_fds_map)) => {
            let mut merged_map = HashMap::new();

            // First, clone all piping FDs
            for (fd, owned_fd) in piping_fds_map {
                merged_map.insert(*fd, owned_fd.try_clone().map_err(|e| ShellError::Io(e))?);
            }

            // Then add/override with redirection FDs
            let redirects_map = setup_redirections_ownedfds(&redirects, env)?;
            for (fd, owned_fd) in redirects_map {
                merged_map.insert(fd, owned_fd);
            }
            Some(merged_map)
        }
        // Case 3: We have piping FDs but no redirects
        (false, Some(piping_fds_map)) => {
            // Clone the existing piping FDs map
            let mut cloned_map = HashMap::new();
            for (fd, owned_fd) in piping_fds_map {
                cloned_map.insert(*fd, owned_fd.try_clone().map_err(|e| ShellError::Io(e))?);
            }
            Some(cloned_map)
        }
        // Case 4: No FDs to merge
        (false, None) => None,
    };

    // 3. Execute the command without waiting
    if !cmd_str.is_empty() {
        match get_command_type(cmd.expand(env).as_str(), env) {
            CommandType::Function(func) => {
                let status = execute(&func, env)?;
                env.set_last_status(status);
                return Ok(CommandResult::Builtin);
            }

            CommandType::Builtin => {
                run_commande(
                    &cmd_str,
                    &all_args,
                    merged_fds.as_ref(),
                    false,
                    HashMap::new(),
                    env,
                    gid,
                )?;
                Ok(CommandResult::Builtin)
            }

            CommandType::External(path) => {
                let mut envs = env.get_environment_only();
                for ass in assignments.clone() {
                    envs.insert(ass.0, ass.1.expand(&env));
                }

                match run_commande(&path, &all_args, merged_fds.as_ref(), true, envs, env, gid)? {
                    CommandResult::Child(child_pid) => {
                        // Return the child PID without waiting
                        Ok(CommandResult::Child(child_pid))
                    }
                    CommandResult::Builtin => Ok(CommandResult::Builtin),
                }
            }
            CommandType::Undefined => {
                Err(ShellError::Exec(format!("Command not found: {}", cmd_str)))
            }
        }
    } else {
        // Handle variable assignments without command
        if !assignments.is_empty() {
            for ass in assignments {
                env.set_local_var(&ass.0, &ass.1.expand(&env));
            }
            return Ok(CommandResult::Builtin);
        }
        return Ok(CommandResult::Builtin);
    }
}
