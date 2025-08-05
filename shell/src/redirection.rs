use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::parser::types::*;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::fd::{FromRawFd, OwnedFd};

use nix::unistd::close;
use nix::unistd::dup;

pub fn setup_redirections_ownedfds(
    redirects: &Vec<Redirect>,
    env: &ShellEnv,
) -> Result<HashMap<u64, OwnedFd>, ShellError> {
    let mut fds_map = HashMap::new();

    for redirect in redirects {
        let fd = redirect.fd.unwrap_or_else(|| match redirect.kind {
            RedirectOp::Read => 0, // stdin
            _ => 1,                // stdout by default
        });

        let target = redirect.target.expand(env);

        // Handle cases like <&- (close FD)
        if target.trim() == "&-" {
            if let Err(e) = close(fd as i32) {
                return Err(ShellError::Exec(format!(
                    "Failed to close fd {}: {}",
                    fd, e
                )));
            }
            continue;
        }

        // Handle cases like 2>&1 (duplicate fds)
        if let Some(stripped) = target.strip_prefix('&') {
            match stripped.parse::<i32>() {
                Ok(target_fd) => match dup(target_fd) {
                    Ok(dup_fd) => {
                        let owned = unsafe { OwnedFd::from_raw_fd(dup_fd) };
                        fds_map.insert(fd, owned);
                    }
                    Err(e) => {
                        return Err(ShellError::Exec(format!(
                            "Failed to duplicate fd {}: {}",
                            target_fd, e
                        )));
                    }
                },
                Err(e) => {
                    return Err(ShellError::Exec(format!(
                        "Invalid file descriptor in redirection: {}",
                        e
                    )));
                }
            }
            continue;
        }

        // Normal file redirection
        let file_result = match redirect.kind {
            RedirectOp::Read => OpenOptions::new().read(true).open(&target),
            RedirectOp::Write => OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&target),
            RedirectOp::Append => OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&target),
            _ => {
                return Err(ShellError::Exec(format!(
                    "Unsupported redirection: {:?}",
                    redirect.kind
                )));
            }
        };

        match file_result {
            Ok(file) => {
                let owned_fd = OwnedFd::from(file);
                fds_map.insert(fd, owned_fd);
            }
            Err(e) => {
                return Err(ShellError::Exec(format!(
                    "Redirection failed for fd {}: {}",
                    fd, e
                )));
            }
        }
    }

    Ok(fds_map)
}
