use crate::config::ENV;
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::exec::*;
use crate::expansion::expand;
use crate::lexer::types::Word;
use crate::parse_redirection::*;
use crate::parser::types::*;
use std::fs::OpenOptions;
use std::io::{self, Read, Write, stdout};
use std::process::Child;
use std::process::Command as ExternalCommand;
use std::process::Stdio;

// use crate::{Redirect, RedirectOp, ShellEnv, ShellError, word_to_string};
// use std::fs::OpenOptions;
use std::os::unix::io::{IntoRawFd, OwnedFd};

pub fn setup_redirections_ownedfds(
    redirects: &Vec<Redirect>,
    env: &ShellEnv,
) -> Result<(Option<OwnedFd>, Option<OwnedFd>, Option<OwnedFd>), ShellError> {
    let mut stdin_fd: Option<OwnedFd> = None;
    let mut stdout_fd: Option<OwnedFd> = None;
    let mut stderr_fd: Option<OwnedFd> = None;

    for redirect in redirects {
        let target_file = word_to_string(&redirect.target, env);

        match redirect.kind {
            RedirectOp::Write => {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        let owned_fd = OwnedFd::from(file);
                        stdout_fd = Some(owned_fd);
                    }
                    Err(e) => {
                        return Err(ShellError::Exec(format!("redirect (write) failed: {}", e)));
                    }
                }
                
            }

            RedirectOp::Append => {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        let owned_fd = OwnedFd::from(file);
                        stdout_fd = Some(owned_fd);
                    }
                    Err(e) => {
                        return Err(ShellError::Exec(format!("redirect (append) failed: {}", e)));
                    }
                }
            }

            RedirectOp::Read => match OpenOptions::new().read(true).open(&target_file) {
                Ok(file) => {
                    let owned_fd = OwnedFd::from(file);
                    stdin_fd = Some(owned_fd);
                }
                Err(e) => {
                    return Err(ShellError::Exec(format!("redirect (read) failed: {}", e)));
                }
            },

            _ => {
                eprintln!("Unsupported redirection: {:?}", redirect.kind);
            }
        }
    }

    Ok((stdin_fd, stdout_fd, stderr_fd))
}
