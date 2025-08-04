use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::*;

use crate::parser::types::*;
use std::fs::OpenOptions;

// use crate::{Redirect, RedirectOp, ShellEnv, ShellError, word_to_string};
// use std::fs::OpenOptions;
use std::os::unix::io::OwnedFd;

use std::collections::HashMap;

pub fn setup_redirections_ownedfds(
    redirects: &Vec<Redirect>,
    env: &ShellEnv,
) -> Result<HashMap<u64, OwnedFd>, ShellError> {
    let mut fds_map = HashMap::new();

    for redirect in redirects {
        let fd = redirect.fd.unwrap_or_else(|| {
            // Default fd for kind:
            match redirect.kind {
                RedirectOp::Read => 0,                       // stdin
                RedirectOp::Write | RedirectOp::Append => 1, // stdout
                _ => 1,                                      // fallback stdout
            }
        });

        let target_file = redirect.target.expand(env);

        let file_result = match redirect.kind {
            RedirectOp::Write => OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&target_file),

            RedirectOp::Append => OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&target_file),

            RedirectOp::Read => OpenOptions::new().read(true).open(&target_file),

            _ => {
                eprintln!("Unsupported redirection: {:?}", redirect.kind);
                continue;
            }
        };

        match file_result {
            Ok(file) => {
                let owned_fd = OwnedFd::from(file);
                fds_map.insert(fd, owned_fd);
            }
            Err(e) => {
                return Err(ShellError::Exec(format!(
                    "redirect failed for fd {}: {}",
                    fd, e
                )));
            }
        }
    }

    Ok(fds_map)
}
