use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::CommandResult;
use crate::exec::build_command;
use nix::fcntl::{FcntlArg, fcntl};
use nix::unistd::close;
use nix::unistd::dup;
use nix::unistd::dup2;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::Command as ExternalCommand;
use std::process::Stdio;
use std::vec;

pub fn run_commande(
    cmd_str: &str,
    args: &[String],
    fds_map: Option<&HashMap<u64, OwnedFd>>,
    use_external: bool,
    assignements: HashMap<String, String>,
    env: &mut ShellEnv,
) -> Result<CommandResult, ShellError> {
    let stdin_fd = fds_map.and_then(|map| map.get(&0));
    let stdout_fd = fds_map.and_then(|map| map.get(&1));
    let stderr_fd = fds_map.and_then(|map| map.get(&2));

    if use_external {
        let mut command = ExternalCommand::new(cmd_str);
        command.args(args);

        // Setup standard fds
        if let Some(fd) = stdin_fd {
            let new_fd = match dup(fd.as_raw_fd()) {
                Ok(val) => val,
                Err(e) => return Err(ShellError::Exec(e.to_string())),
            };
            command.stdin(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
        } else {
            command.stdin(Stdio::inherit());
        }

        if let Some(fd) = stdout_fd {
            let new_fd = match dup(fd.as_raw_fd()) {
                Ok(val) => val,
                Err(e) => return Err(ShellError::Exec(e.to_string())),
            };
            command.stdout(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
        } else {
            command.stdout(Stdio::inherit());
        }

        if let Some(fd) = stderr_fd {
            let new_fd = match dup(fd.as_raw_fd()) {
                Ok(val) => val,
                Err(e) => return Err(ShellError::Exec(e.to_string())),
            };
            command.stderr(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
        } else {
            command.stderr(Stdio::inherit());
        }

        // Setup arbitrary fds > 2
        if let Some(map) = fds_map {
            let extra_fds: Vec<(i32, i32)> = map
                .iter()
                .filter(|(fd, _)| fd > &&2)
                .map(|(&target_fd, owned_fd)| (target_fd as i32, owned_fd.as_raw_fd()))
                .collect();
            // Apply those via pre_exec (runs in child before exec)
            unsafe {
                command.pre_exec(move || {
                    for (target_fd, source_fd) in &extra_fds {
                        // Check if source_fd is valid
                        if fcntl(*source_fd, FcntlArg::F_GETFD).is_err() {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Invalid source fd: {}", source_fd),
                            ));
                        }

                        // Proceed with dup2
                        dup2(*source_fd, *target_fd).map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("dup2 failed: {}", e),
                            )
                        })?;
                    }
                    Ok(())
                });
            }
        }
        return command
            .envs(assignements)
            .spawn()
            .map(CommandResult::Child)
            .map_err(|e| ShellError::Exec(format!("Failed to spawn {}: {}", cmd_str, e)));
    } else {
        let com = build_command(&cmd_str.to_owned(), args.to_vec(), vec![], None);
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
