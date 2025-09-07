use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::CommandResult;
use crate::exec::build_command;
use nix::fcntl::{FcntlArg, fcntl};
use nix::unistd::getpid;
use nix::unistd::setpgid;
use nix::unistd::tcsetpgrp;
use nix::unistd::{ForkResult, Pid, close, dup, dup2, execve, fork};
use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::vec;

fn execute_external_with_fork(
    cmd_path: &str, // This is now the full path to the executable
    args: &[String],
    fds_map: Option<&HashMap<u64, OwnedFd>>,
    assignments: &HashMap<String, String>,
) -> Result<CommandResult, ShellError> {
    // Prepare command and arguments as CStrings
    let cmd_cstring = CString::new(cmd_path)
        .map_err(|e| ShellError::Exec(format!("Invalid command path: {}", e)))?;

    let mut arg_cstrings = Vec::new();
    // argv[0] should be the command name (not necessarily the full path)
    let cmd_name = std::path::Path::new(cmd_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(cmd_path);
    let cmd_name_cstring = CString::new(cmd_name)
        .map_err(|e| ShellError::Exec(format!("Invalid command name: {}", e)))?;
    arg_cstrings.push(cmd_name_cstring);

    for arg in args {
        let arg_cstring = CString::new(arg.as_str())
            .map_err(|e| ShellError::Exec(format!("Invalid argument: {}", e)))?;
        arg_cstrings.push(arg_cstring);
    }

    // Prepare environment variables
    let mut env_vars = Vec::new();

    // Start with current environment
    for (key, value) in env::vars() {
        // Skip variables that will be overridden
        if !assignments.contains_key(&key) {
            let env_string = format!("{}={}", key, value);
            let env_cstring = CString::new(env_string)
                .map_err(|e| ShellError::Exec(format!("Invalid environment variable: {}", e)))?;
            env_vars.push(env_cstring);
        }
    }

    // Add assignments
    for (key, value) in assignments {
        let env_string = format!("{}={}", key, value);
        let env_cstring = CString::new(env_string)
            .map_err(|e| ShellError::Exec(format!("Invalid assignment: {}", e)))?;
        env_vars.push(env_cstring);
    }

    // Get file descriptors for standard streams
    let stdin_fd = fds_map.and_then(|map| map.get(&0));
    let stdout_fd = fds_map.and_then(|map| map.get(&1));
    let stderr_fd = fds_map.and_then(|map| map.get(&2));

    // Duplicate file descriptors that need to be preserved in the child
    let stdin_new_fd = if let Some(fd) = stdin_fd {
        Some(
            dup(fd.as_raw_fd())
                .map_err(|e| ShellError::Exec(format!("Failed to dup stdin: {}", e)))?,
        )
    } else {
        None
    };

    let stdout_new_fd = if let Some(fd) = stdout_fd {
        Some(
            dup(fd.as_raw_fd())
                .map_err(|e| ShellError::Exec(format!("Failed to dup stdout: {}", e)))?,
        )
    } else {
        None
    };

    let stderr_new_fd = if let Some(fd) = stderr_fd {
        Some(
            dup(fd.as_raw_fd())
                .map_err(|e| ShellError::Exec(format!("Failed to dup stderr: {}", e)))?,
        )
    } else {
        None
    };

    // Prepare extra file descriptors (fds > 2)
    let extra_fds: Vec<(i32, i32)> = if let Some(map) = fds_map {
        map.iter()
            .filter(|(fd, _)| **fd > 2)
            .map(|(&target_fd, owned_fd)| (target_fd as i32, owned_fd.as_raw_fd()))
            .collect()
    } else {
        Vec::new()
    };

    // Fork the process
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // Parent process - clean up duplicated fds and return child PID
            if let Some(fd) = stdin_new_fd {
                let _ = close(fd);
            }
            if let Some(fd) = stdout_new_fd {
                let _ = close(fd);
            }
            if let Some(fd) = stderr_new_fd {
                let _ = close(fd);
            }
            Ok(CommandResult::Child(child))
        }

        Ok(ForkResult::Child) => {
            // Child process - setup file descriptors and exec
            let child_pid = getpid();
            // Make child leader of new PGID
            let _ = setpgid(child_pid, child_pid);


            // Setup standard file descriptors
            if let Some(new_fd) = stdin_new_fd {
                if dup2(new_fd, 0).is_err() {
                    std::process::exit(1); // Exit child on error
                }
                let _ = close(new_fd); // Close the duplicated fd
            }

            if let Some(new_fd) = stdout_new_fd {
                if dup2(new_fd, 1).is_err() {
                    std::process::exit(1);
                }
                let _ = close(new_fd);
            }

            if let Some(new_fd) = stderr_new_fd {
                if dup2(new_fd, 2).is_err() {
                    std::process::exit(1);
                }
                let _ = close(new_fd);
            }

            // Setup extra file descriptors
            for (target_fd, source_fd) in extra_fds {
                // Validate source fd
                if fcntl(source_fd, FcntlArg::F_GETFD).is_err() {
                    std::process::exit(1);
                }

                if dup2(source_fd, target_fd).is_err() {
                    std::process::exit(1);
                }
            }

            // Convert CStrings to CStr references for execve
            let argv: Vec<&CStr> = arg_cstrings.iter().map(|s| s.as_c_str()).collect();
            let envp: Vec<&CStr> = env_vars.iter().map(|s| s.as_c_str()).collect();

            // Execute the command using the full path
            let _ = execve(&cmd_cstring, &argv, &envp);

            // If execve returns, it failed
            std::process::exit(127); // Standard exit code for command not found
        }

        Err(e) => {
            // Clean up duplicated fds on fork failure
            if let Some(fd) = stdin_new_fd {
                let _ = close(fd);
            }
            if let Some(fd) = stdout_new_fd {
                let _ = close(fd);
            }
            if let Some(fd) = stderr_new_fd {
                let _ = close(fd);
            }
            Err(ShellError::Exec(format!("Fork failed: {}", e)))
        }
    }
}

pub fn run_commande(
    cmd_str: &str,
    args: &[String],
    fds_map: Option<&HashMap<u64, OwnedFd>>,
    use_external: bool,
    assignements: HashMap<String, String>,
    env: &mut ShellEnv,
) -> Result<CommandResult, ShellError> {
    if use_external {
        // cmd_str is now the full path to the external command
        return execute_external_with_fork(cmd_str, args, fds_map, &assignements);
    } else {
        // Handle builtin commands (unchanged)
        let com = build_command(
            &cmd_str.to_owned(),
            args.to_vec(),
            vec![],
            None,
            env.clone(),
        );
        let mut backups: Option<Vec<(u64, i32)>> = None;
        if let Some(map) = fds_map {
            backups = Some(
                map.iter()
                    .filter_map(|(&fd, _)| match dup(fd as i32) {
                        Ok(dup_fd) => Some((fd, dup_fd)),
                        Err(err) => {
                            eprintln!("Failed to duplicate fd {}: {}", fd, err);
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            );

            for (&target_fd, owned_fd) in map {
                let source_fd = owned_fd.as_raw_fd();
                dup2(source_fd, target_fd as i32).map_err(|e| {
                    ShellError::Exec(format!("dup2 failed for fd {}: {}", target_fd, e))
                })?;
            }
        }
        match com {
            Some(val) => {
                val.execute(env)?;
                if let Some(back) = backups {
                    for (fd, backup) in back {
                        dup2(backup, fd as i32).ok();
                        close(backup).ok();
                    }
                }
                return Ok(CommandResult::Builtin);
            }
            None => {
                return Err(ShellError::Exec(format!(
                    "Internal command not found: {}",
                    cmd_str
                )));
            }
        }
    }
}
