use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::CommandResult;
use crate::exec::CommandType;
use crate::exec::execute;
use crate::exec::get_command_type;
use crate::executor::run_commande::run_commande;
use crate::expansion::expand_and_split;
use crate::features::jobs;
use crate::lexer::types::Word;
use crate::redirection::setup_redirections_ownedfds;
use crate::types::Redirect;
use nix::sys::signal::Signal;
use nix::sys::signal::signal;
use nix::unistd::Pid;
use nix::unistd::getpgrp;
use nix::unistd::setpgid;
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
    gid: Option<Pid>,
    is_backgound: bool,
) -> Result<i32, ShellError> {
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
                // Note: This assumes OwnedFd implements Clone or you have a way to duplicate it
                // If OwnedFd doesn't implement Clone, you might need to use a different approach
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
    // 3. Execute the command
    if !cmd_str.is_empty() {
        match get_command_type(cmd.expand(env).as_str(), env) {
            CommandType::Function(func) => {
                let status = execute(&func, env)?;
                env.set_last_status(status);
                return Ok(status);
            }

            CommandType::Builtin => {
                run_commande(
                    &cmd_str,
                    &all_args,
                    merged_fds.as_ref(),
                    false,
                    HashMap::new(),
                    env,
                )?;
                Ok(0)
            }

            CommandType::External(path) => {
                let tty = File::open("/dev/tty").unwrap();
                let fd = tty.into_raw_fd();

                let mut envs = env.get_environment_only();
                for ass in assignments.clone() {
                    envs.insert(ass.0, ass.1.expand(&env));
                }
                let status =
                    match run_commande(&path, &all_args, merged_fds.as_ref(), true, envs, env)? {
                        CommandResult::Child(mut child_pid) => {
                            let mut g = match gid {
                                Some(val) => val,
                                None => child_pid,
                            };

                            // set the process group id to the current child
                            setpgid(child_pid, child_pid).unwrap();

                            println!("{}", env.current_command);
                            // Create a new job and add it to the jobs class

                            let new_job = jobs::Job::new(
                                child_pid,
                                child_pid,
                                env.jobs.size,
                                jobs::JobStatus::Running,
                                cmd_str,
                            );
                            env.jobs.add_job(new_job);

                            if (!is_backgound) {
                                // Get shell PGID
                                let shell_pgid = getpgrp();

                                // Temporarily ignore SIGTTOU so parent doesn't suspend
                                let old = unsafe {
                                    signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn)
                                }
                                .unwrap();
                                // Give terminal control to child
                                tcsetpgrp(fd, child_pid).unwrap();
                                // Restore SIGTTOU handler
                                unsafe { signal(Signal::SIGTTOU, old).unwrap() };

                                // Wait for child to finish
                                let exitcode = match nix::sys::wait::waitpid(
                                    child_pid,
                                    Some(nix::sys::wait::WaitPidFlag::WUNTRACED),
                                ) {
                                    Ok(wait_status) => match wait_status {
                                        nix::sys::wait::WaitStatus::Exited(_, code) => code,
                                        nix::sys::wait::WaitStatus::Signaled(_, _, _) => 1,
                                        nix::sys::wait::WaitStatus::Stopped(_, _) => {
                                            println!("[{}]+ Stopped", child_pid);
                                            1
                                        }
                                        _ => 1,
                                    },
                                    Err(_) => 1,
                                };

                                // Return terminal control to shell
                                let old = unsafe {
                                    signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn)
                                }
                                .unwrap();
                                tcsetpgrp(fd, shell_pgid).ok();
                                unsafe { signal(Signal::SIGTTOU, old).unwrap() };

                                exitcode
                            } else {
                                0
                            }
                        }
                        CommandResult::Builtin => 0,
                    };

                //  tcsetpgrp(fd, getpid()).unwrap();

                env.set_last_status(status);
                Ok(status)
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
            return Ok(0);
        }
        return Ok(0);
    }
}
