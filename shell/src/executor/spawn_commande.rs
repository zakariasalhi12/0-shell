use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::CommandResult;
use crate::exec::CommandType;
use crate::exec::execute;
use crate::exec::get_command_type;
use crate::executor::run_commande::run_commande;
use crate::expansion::expand_and_split;
use crate::lexer::types::Word;
use crate::redirection::setup_redirections_ownedfds;
use crate::types::Redirect;
use libc::{SIGINT as libc_SIGINT, SIGTERM as libc_SIGTERM};
use std::collections::HashMap;
use std::os::unix::io::OwnedFd;
use std::vec;

pub fn invoke_command(
    cmd: &Word,
    args: &Vec<Word>,
    assignments: &Vec<(String, Word)>,
    redirects: &Vec<Redirect>,
    env: &mut ShellEnv,
    piping_fds: Option<&HashMap<u64, OwnedFd>>,
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
                let mut envs = env.get_environment_only();
                for ass in assignments.clone() {
                    envs.insert(ass.0, ass.1.expand(&env));
                }
                use signal_hook::iterator::Signals;
                let status =
                    match run_commande(&path, &all_args, merged_fds.as_ref(), true, envs, env)? {
                        CommandResult::Child(mut child) => {
                            let mut signals = Signals::new(&[libc_SIGINT, libc_SIGTERM])?;

                            for sig in signals.pending() {
                                if sig == libc_SIGINT {
                                    nix::sys::signal::kill(child, nix::sys::signal::Signal::SIGINT)
                                        .map_err(|e| ShellError::Io(std::io::Error::from(e)))?;
                                }
                            }

                            match nix::sys::wait::waitpid(child, None) {
                                Ok(wait_status) => match wait_status {
                                    nix::sys::wait::WaitStatus::Exited(_, code) => code,
                                    nix::sys::wait::WaitStatus::Signaled(_, _, _) => 1,
                                    _ => 1,
                                },
                                Err(_) => 1,
                            }
                        }
                        CommandResult::Builtin => 0,
                    };

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
