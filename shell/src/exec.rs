use crate::PathBuf;
use crate::ShellCommand;
use crate::redirection::setup_redirections_ownedfds;
use nix::fcntl::{FcntlArg, fcntl};
use nix::unistd::dup;
use nix::unistd::pipe;
use std::char;
use std::os::fd::AsFd;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::vec;

use crate::commands::{
    cat::Cat, cd::Cd, cp::Cp, echo::Echo, export::Export, ls::Ls, mkdir::Mkdir, mv::Mv, pwd::Pwd,
    rm::Rm, typ::Type,
};
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::expansion::expand_and_split;
use crate::lexer::types::Word;
use crate::parser::types::*;
use std::process::Child;
use std::process::Command as ExternalCommand;
use std::process::Stdio;

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            let mut child = execute_commande(cmd, args, assignments, redirects, env, None)?;
            Ok(child)
        }
        AstNode::Pipeline(nodes) => {
            if nodes.is_empty() {
                return Ok(0);
            }

            if nodes.len() == 1 {
                return execute(&nodes[0], env);
            }

            let mut children: Vec<Child> = Vec::new();
            let mut prev_read: Option<OwnedFd> = None;

            for (i, node) in nodes.iter().enumerate() {
                let is_last = i == nodes.len() - 1;

                if let AstNode::Command {
                    cmd,
                    args,
                    redirects,
                    ..
                } = node
                {
                    let cmd_str: String = cmd.expand(&env);
                    let all_args: Vec<String> = args.iter().map(|w| w.expand(env)).collect();

                    // Use the read end of the previous pipe as stdin
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

                    let stderr = Stdio::inherit();
                    // let use_external = should_use_external_for_pipeline(&cmd_str);

                    let fds_map = if redirects.is_empty() {
                        let mut map: HashMap<u64, OwnedFd> = HashMap::new();
                        if let Some(stdi) = stdin {
                            map.insert(0, stdi);
                        }
                        if let Some(stdo) = write_end {
                            map.insert(1, stdo);
                        }
                        Some(map)
                    } else {
                        Some(setup_redirections_ownedfds(&redirects, env)?)
                    };

                    let stat =
                        execute_commande(cmd, args, &vec![], redirects, env, fds_map.as_ref())?;
                    // children.push(char);
                    env.set_last_status(stat);
                    prev_read = read_end; // becomes stdin for next command
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }

            // let mut last_status = 0;
            // for mut child in children {
            //     let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
            //     last_status = status;
            // }

            // env.set_last_status(last_status);
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
        AstNode::Background(node) => {
            // Execute node in background (basic implementation)
            // In a full implementation, this would:
            // - Fork the process
            // - Add to job control
            // - Return immediately
            let status = execute(node, env)?;
            env.set_last_status(status);
            Ok(status)
        }
        AstNode::Subshell(node) => {
            // Execute node in a subshell (basic implementation)
            // In a full implementation, this would:
            // - Fork the process
            // - Create a new environment
            // - Execute the node
            // - Return the status
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
                println!("[exec] Group redirects: {:?}", redirects);
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // Execute condition, then then_branch or else_branch
            let condition_status = execute(condition, env)?;

            if condition_status == 0 {
                // Condition succeeded, execute then branch
                let status = execute(then_branch, env)?;
                env.set_last_status(status);
                Ok(status)
            } else {
                // Condition failed, execute else branch if it exists
                match else_branch {
                    Some(else_node) => {
                        let status = execute(else_node, env)?;
                        env.set_last_status(status);
                        Ok(status)
                    }
                    None => {
                        env.set_last_status(condition_status);
                        Ok(condition_status)
                    }
                }
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
        AstNode::For { var, values, body } => {
            // Execute for loop
            let mut last_status = 0;

            for value in values {
                // Set the loop variable
                // env.set_var(var, value);

                last_status = execute(body, env)?;
                // Continue loop regardless of body status
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        _ => Ok(0), // AstNode::Case { word, arms } => {
                    //     // Execute case statement
                    //     let word_value = word_to_string(
                    //         &Word {
                    //             parts: vec![crate::lexer::types::WordPart::Literal(word.clone())],
                    //             quote: crate::lexer::types::QuoteType::None,
                    //         },
                    //         env,
                    //     );
                    //     let mut last_status = 0;
                    //     let mut matched = false;

                    //     for (patterns, body) in arms {
                    //         for pattern in patterns {
                    //             if pattern == &word_value {
                    //                 last_status = execute(body, env)?;
                    //                 matched = true;
                    //                 break;
                    //             }
                    //         }
                    //         if matched {
                    //             break;
                    //         }
                    //     }

                    //     env.set_last_status(last_status);
                    //     Ok(last_status)
                    // }
                    // AstNode::FunctionDef { name, body } => {
                    //     // Register function in environment
                    //     let func_name = word_to_string(name, env);
                    //     env.set_func(func_name, body.as_ref().clone());
                    //     env.set_last_status(0);
                    //     Ok(0)
                    // }
                    // AstNode::ArithmeticCommand(expr) => {
                    //     // Evaluate arithmetic expression
                    //     // For now, return 0 - full implementation would evaluate the expression
                    //     println!("[exec] ArithmeticCommand: {:?}", expr);
                    //     env.set_last_status(0);
                    //     Ok(0)
                    // }
    }
}

pub fn build_command(
    cmd: &String,
    args: Vec<String>,
    opts: Vec<String>,
    stdout: Option<OwnedFd>,
) -> Option<Box<dyn ShellCommand>> {
    match cmd.as_str() {
        "echo" => Some(Box::new(Echo::new(args, stdout))),
        "cd" => Some(Box::new(Cd::new(args))),
        "ls" => Some(Box::new(Ls::new(args, opts))),
        "pwd" => Some(Box::new(Pwd::new(args))),
        "cat" => Some(Box::new(Cat::new(args))),
        "cp" => Some(Box::new(Cp::new(args, opts))),
        "rm" => Some(Box::new(Rm::new(args, opts))),
        "mv" => Some(Box::new(Mv::new(args))),
        "mkdir" => Some(Box::new(Mkdir::new(args, opts))),
        "export" => Some(Box::new(Export::new(args))),
        "type" => Some(Box::new(Type::new(args))),
        "exit" => {
            std::process::exit(0);
        }
        _ => None,
    }
}
pub enum CommandResult {
    Child(Child),
    Builtin,
}

fn should_use_external_for_pipeline(cmd: &str) -> bool {
    matches!(cmd, "ls" | "cat" | "grep")
}

use nix::unistd::{close, dup2};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;

pub fn execute_command_with_stdio(
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
        // let env_result = ENV.lock();
        // if let Ok(env_map) = env_result {
        // if let Some(full_path) = env_map.get(cmd_str) {
        let mut command = ExternalCommand::new(cmd_str);
        command.args(args);

        // Setup standard fds
        if let Some(fd) = stdin_fd {
            let new_fd = dup(fd.as_raw_fd()).unwrap();
            command.stdin(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
        } else {
            command.stdin(Stdio::inherit());
        }

        if let Some(fd) = stdout_fd {
            let new_fd = dup(fd.as_raw_fd()).unwrap();
            command.stdout(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
        } else {
            command.stdout(Stdio::inherit());
        }

        if let Some(fd) = stderr_fd {
            let new_fd = dup(fd.as_raw_fd()).unwrap();
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
        // }
        // }
    } else {
        // Internal command: temporarily redirect fds in current process

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

    Err(ShellError::Exec(format!("Command not found: {}", cmd_str)))
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
        "echo" | "cd" | "pwd" | "cat" | "cp" | "rm" | "mv" | "mkdir" | "export" | "exit"
        | "type" => CommandType::Builtin,
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

pub fn execute_commande(
    cmd: &Word,
    args: &Vec<Word>,
    assignments: &Vec<(String, Word)>,
    redirects: &Vec<Redirect>,
    env: &mut ShellEnv,
    mut piping_fds: Option<&HashMap<u64, OwnedFd>>,
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

    //  let merged_fds = if !redirects.is_empty() {
    //     match setup_redirections_ownedfds(&redirects, env) {
    //         Ok(redirects_map) => {
    //             if let Some(piping_fds_map) = piping_fds {
    //                 // Merge the two maps
    //                 let mut merged = piping_fds_map.clone();
    //                 merged.extend(redirects_map);
    //                 Some(merged)
    //             } else {
    //                 Some(redirects_map).as_ref()
    //             }
    //         }
    //         Err(e) => return Err(e),
    //     }
    // } else {
    //     // No redirects, just use piping_fds as is
    //     piping_fds
    // };

    if !cmd_str.is_empty() {
        match get_command_type(cmd.expand(env).as_str(), env) {
            CommandType::Function(func) => {
                let status = execute(&func, env)?;
                env.set_last_status(status);
                return Ok(status);
            }

            CommandType::Builtin => {
                execute_command_with_stdio(
                    &cmd_str,
                    &all_args,
                    piping_fds,
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

                let status = match execute_command_with_stdio(
                    &path, &all_args, piping_fds, true, envs, env,
                )? {
                    CommandResult::Child(mut child) => {
                        child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1)
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
        if !assignments.is_empty() {
            for ass in assignments {
                env.set_local_var(&ass.0, &ass.1.expand(&env));
            }
            return Ok(0);
        }
        return Ok(0);
    }
}
