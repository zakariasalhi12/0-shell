use crate::PathBuf;
use crate::ShellCommand;
use crate::commands::bg::Bg;
use crate::commands::exit::Exit;
use crate::commands::fg::Fg;
use crate::commands::jobs::Jobs;
use crate::commands::kill::Kill;
use crate::executor::spawn_commande::invoke_command;
use nix::unistd::Pid;
use nix::unistd::pipe;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::vec;

use crate::commands::{
    cd::Cd, cp::Cp, echo::Echo, export::Export, mkdir::Mkdir, mv::Mv, pwd::Pwd, rm::Rm, typ::Type,
};
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::parser::types::*;

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    env.current_command = ast.to_text(env);
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            let child = invoke_command(
                cmd,
                args,
                assignments,
                redirects,
                env,
                None,
                &mut None,
                false,
            )?;
            env.set_last_status(child);
            Ok(child)
        }
        AstNode::Pipeline(nodes) => {
            if nodes.is_empty() {
                return Ok(0);
            }

            if nodes.len() == 1 {
                return execute(&nodes[0], env);
            }

            let mut prev_read: Option<OwnedFd> = None;
            let mut gid = Option::<Pid>::None;

            for (i, node) in nodes.iter().enumerate() {
                let is_last = i == nodes.len() - 1;

                if let AstNode::Command {
                    cmd,
                    args,
                    redirects,
                    ..
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

                    let stat = invoke_command(
                        cmd,
                        args,
                        &vec![],
                        redirects,
                        env,
                        fds_map.as_ref(),
                        &mut gid,
                        false,
                    )?;
                    env.set_last_status(stat);
                    prev_read = read_end; // becomes stdin for next command
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }
            Ok(env.get_last_status())
        }

        AstNode::Sequence(nodes) => {
            let mut last_status = 0;

            for node in nodes {
                last_status = execute(node, env)?;
                // Continue execution even if a command fails
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::And(left, right) => {
            // Execute left, if success then right
            let left_status = execute(left, env)?;
            if left_status == 0 {
                let right_status = execute(right, env)?;
                env.set_last_status(right_status);
                Ok(right_status)
            } else {
                env.set_last_status(left_status);
                Ok(left_status)
            }
        }
        AstNode::Or(left, right) => {
            // Execute left, if fail then right
            let left_status = execute(left, env)?;
            if left_status != 0 {
                let right_status = execute(right, env)?;
                env.set_last_status(right_status);
                Ok(right_status)
            } else {
                env.set_last_status(left_status);
                Ok(left_status)
            }
        }
        AstNode::Not(node) => {
            // Execute node, invert status
            let status = execute(node, env)?;
            let inverted_status = if status == 0 { 1 } else { 0 };
            env.set_last_status(inverted_status);
            Ok(inverted_status)
        }
        AstNode::Background(node) => match **node {
            AstNode::Command {
                ref cmd,
                ref args,
                ref assignments,
                ref redirects,
            } => {
                let child = invoke_command(
                    cmd,
                    args,
                    assignments,
                    redirects,
                    env,
                    None,
                    &mut None,
                    true,
                )?;
                env.set_last_status(child);
                Ok(child)
            }
            _ => {
                let status = execute(&node, env)?;
                env.set_last_status(status);
                Ok(status)
            }
        },
        AstNode::Subshell(node) => {
            let status = execute(node, env)?;
            env.set_last_status(status);
            Ok(status)
        }
        AstNode::Group {
            commands,
            redirects,
        } => {
            // Execute group of commands, handle redirects
            let mut last_status = 0;

            for command in commands {
                last_status = execute(command, env)?;
            }

            // Handle redirects (basic implementation)
            if !redirects.is_empty() {
                // println!("[exec] Group redirects: {:?}", redirects);
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::If {
            condition,
            then_branch,
            elif,
            else_branch,
        } => {
            // Execute condition
            let condition_status = execute(condition, env)?;

            if condition_status == 0 {
                // Condition succeeded, execute then branch
                let status = execute(then_branch, env)?;
                env.set_last_status(status);
                Ok(status)
            } else {
                // Try elif branches in order
                let mut matched = false;
                let mut status = condition_status;

                for (elif_cond, elif_body) in elif.iter() {
                    let elif_cond_status = execute(elif_cond, env)?;
                    if elif_cond_status == 0 {
                        status = execute(elif_body, env)?;
                        matched = true;
                        break;
                    } else {
                        status = elif_cond_status;
                    }
                }

                if !matched {
                    // Execute else branch if present
                    if let Some(else_node) = else_branch {
                        status = execute(else_node, env)?;
                    }
                }

                env.set_last_status(status);
                Ok(status)
            }
        }
        AstNode::While { condition, body } => {
            // Execute while loop
            let mut last_status = 0;

            loop {
                let condition_status = execute(condition, env)?;
                if condition_status != 0 {
                    // Condition failed, exit loop
                    break;
                }

                last_status = execute(body, env)?;
                // Continue loop regardless of body status
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::Until { condition, body } => {
            // Execute until loop (opposite of while)
            let mut last_status = 0;

            loop {
                let condition_status = execute(condition, env)?;
                if condition_status == 0 {
                    // Condition succeeded, exit loop
                    break;
                }

                last_status = execute(body, env)?;
                // Continue loop regardless of body status
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::For {
            var: _,
            values,
            body,
        } => {
            // Execute for loop
            let mut last_status = 0;

            for _ in values {
                // Set the loop variable
                // env.set_var(var, value);

                last_status = execute(body, env)?;
                // Continue loop regardless of body status
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        _ => Ok(0),
    }
}

pub fn build_command(
    cmd: &String,
    args: Vec<String>,
    opts: Vec<String>,
    stdout: Option<OwnedFd>,
    shellenv: ShellEnv,
) -> Option<Box<dyn ShellCommand>> {
    match cmd.as_str() {
        "echo" => Some(Box::new(Echo::new(args, stdout))),
        "cd" => Some(Box::new(Cd::new(args))),
        "pwd" => Some(Box::new(Pwd::new(args))),
        "cp" => Some(Box::new(Cp::new(args, opts))),
        "rm" => Some(Box::new(Rm::new(args, opts))),
        "mv" => Some(Box::new(Mv::new(args))),
        "mkdir" => Some(Box::new(Mkdir::new(args, opts))),
        "export" => Some(Box::new(Export::new(args))),
        "type" => Some(Box::new(Type::new(args))),
        "fg" => Some(Box::new(Fg::new(args))),
        "exit" => Some(Box::new(Exit::new(args, opts))),
        "jobs" => Some(Box::new(Jobs::new(args))),
        "kill" => Some(Box::new(Kill::new(args))),
        "bg" => Some(Box::new(Bg::new(args))),
        _ => None,
    }
}
pub enum CommandResult {
    Child(Pid),
    Builtin(i32), // Status
}

pub enum CommandType {
    Builtin,
    External(String),
    Function(AstNode),
    Undefined,
}

pub fn get_command_type(cmd: &str, env: &mut ShellEnv) -> CommandType {
    if let Some(func) = env.get_func(&cmd) {
        return CommandType::Function(func.clone());
    }

    match cmd {
        "echo" | "cd" | "pwd" | "cp" | "rm" | "mv" | "mkdir" | "export" | "exit" | "type"
        | "fg" | "jobs" | "kill" | "bg" => CommandType::Builtin,
        _ => match env.get("PATH") {
            Some(bin_path) => {
                let paths: Vec<&str> = bin_path.split(':').collect();
                for path in paths {
                    let path = PathBuf::from(path);
                    let full_path = path.join(cmd);
                    if full_path.exists() {
                        return CommandType::External(full_path.to_string_lossy().to_string());
                    }
                }
                return CommandType::Undefined;
            }
            None => return CommandType::Undefined,
        },
    }
}
