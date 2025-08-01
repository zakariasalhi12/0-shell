use crate::ShellCommand;
use nix::unistd::dup;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

use crate::builtins::try_builtin;
use nix::unistd::{pipe, read, write};

use crate::commands::{
    cat::Cat, cd::Cd, cp::Cp, echo::Echo, export::Export, ls::Ls, mkdir::Mkdir, mv::Mv, pwd::Pwd,
    rm::Rm,
};
use crate::config::ENV;
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::expansion::expand;
use crate::lexer::types::Word;
use crate::parser::types::*;
use std::io::{self, Read, Write, stdout};
use std::process::Child;
use std::process::Command as ExternalCommand;
use std::process::Stdio;

fn word_to_string(word: &crate::lexer::types::Word, env: &ShellEnv) -> String {
    // Expand and join all parts (for now, just join literals)
    let mut result = String::new();
    for part in &word.parts {
        match part {
            crate::lexer::types::WordPart::Literal(s) => result.push_str(s),
            crate::lexer::types::WordPart::VariableSubstitution(var) => {
                if let Some(val) = env.get_var(var) {
                    result.push_str(val);
                }
            }
            // TODO: handle ArithmeticSubstitution, CommandSubstitution
            _ => {}
        }
    }
    result
}

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            // println!("{}", ast);
            // 1. Expand command and args
            let cmd_str = word_to_string(cmd, env);
            let all_args: Vec<String> = args.iter().map(|w| word_to_string(w, env)).collect();

            let opts: Vec<String> = all_args
                .iter()
                .filter(|v| v.starts_with('-'))
                .cloned()
                .collect();

            let arg_strs: Vec<String> = all_args
                .iter()
                .filter(|v| !v.starts_with('-'))
                .cloned()
                .collect();

            // 2. Handle assignments
            if !assignments.is_empty() {
                for ass in assignments.clone() {
                    let value = word_to_string(
                        &Word {
                            parts: ass.1,
                            quote: crate::lexer::types::QuoteType::None,
                        },
                        env,
                    );
                    env.set_var(&ass.0, &value);
                }
            }

            // 3. Handle redirects (basic implementation)
            if !redirects.is_empty() {
                // For now, just log redirects - full implementation would require file handling
                println!("[exec] Redirects: {:?}", redirects);
            }

            // 4. Check for built-in
            if !cmd_str.is_empty() {
                // Check if a function in envirement functions
                if let Some(func) = env.get_func(&cmd_str) {
                    let body = func.clone(); // <- Clone here
                    let status = execute(&body, env)?; // <- Now safe to mutably borrow env
                    env.set_last_status(status);
                    return Ok(status);
                }

                let command = build_command(&cmd_str, arg_strs.clone(), opts);
                match command {
                    Some(val) => {
                        let res = val.execute();
                        match res {
                            Ok(_) => {
                                env.set_last_status(0);
                                Ok(0)
                            }
                            Err(e) => {
                                eprintln!("{e}");
                                env.set_last_status(1);
                                Ok(1)
                            }
                        }
                    }
                    None => {
                        // 5. Try to run as external command
                        let env_result = ENV.lock();
                        if let Ok(mut env_map) = env_result {
                            // Get the full path from your environment map
                            if let Some(full_path) = env_map.get(&cmd_str) {
                                println!("Found command at: {}", full_path);

                                // Use the full path instead of just the command name
                                let mut child =
                                    match ExternalCommand::new(full_path) // Use full_path here
                                        .args(&all_args)
                                        .stdin(Stdio::inherit())
                                        .stdout(Stdio::inherit())
                                        .stderr(Stdio::inherit())
                                        .spawn()
                                    {
                                        Ok(child) => child,
                                        Err(e) => {
                                            eprintln!(
                                                "{}: command failed to execute: {}",
                                                full_path, e
                                            );
                                            env.set_last_status(127);
                                            return Ok(127);
                                        }
                                    };

                                let status =
                                    child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
                                env.set_last_status(status);
                                Ok(status)
                            } else {
                                // Command not found in your environment map
                                eprintln!("{}: command not found", cmd_str);
                                env.set_last_status(127);
                                return Ok(127);
                            }
                        } else {
                            return Err(ShellError::Exec(cmd_str));
                        }
                    }
                }
            } else {
                Ok(0)
            }
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
                    let cmd_str = word_to_string(cmd, env);
                    let all_args: Vec<String> =
                        args.iter().map(|w| word_to_string(w, env)).collect();

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
                    let use_external = should_use_external_for_pipeline(&cmd_str);

                    let child = execute_command_with_stdio(
                        &cmd_str,
                        &all_args,
                        stdin,
                        write_end,
                        stderr,
                        i == 0,
                        is_last,
                        env,
                        use_external,
                    )?;

                    children.push(child);
                    prev_read = read_end; // becomes stdin for next command
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }

            let mut last_status = 0;
            for mut child in children {
                let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
                last_status = status;
            }

            env.set_last_status(last_status);
            Ok(last_status)
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
                env.set_var(var, value);

                last_status = execute(body, env)?;
                // Continue loop regardless of body status
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::Case { word, arms } => {
            // Execute case statement
            let word_value = word_to_string(
                &Word {
                    parts: vec![crate::lexer::types::WordPart::Literal(word.clone())],
                    quote: crate::lexer::types::QuoteType::None,
                },
                env,
            );
            let mut last_status = 0;
            let mut matched = false;

            for (patterns, body) in arms {
                for pattern in patterns {
                    if pattern == &word_value {
                        last_status = execute(body, env)?;
                        matched = true;
                        break;
                    }
                }
                if matched {
                    break;
                }
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::FunctionDef { name, body } => {
            // Register function in environment
            let func_name = word_to_string(name, env);
            env.set_func(func_name, body.as_ref().clone());
            env.set_last_status(0);
            Ok(0)
        }
        AstNode::ArithmeticCommand(expr) => {
            // Evaluate arithmetic expression
            // For now, return 0 - full implementation would evaluate the expression
            println!("[exec] ArithmeticCommand: {:?}", expr);
            env.set_last_status(0);
            Ok(0)
        }
    }
}

pub fn build_command(
    cmd: &String,
    args: Vec<String>,
    opts: Vec<String>,
) -> Option<Box<dyn ShellCommand>> {
    match cmd.as_str() {
        "echo" => Some(Box::new(Echo::new(args))),
        "cd" => Some(Box::new(Cd::new(args))),
        "ls" => Some(Box::new(Ls::new(args, opts))),
        "pwd" => Some(Box::new(Pwd::new(args))),
        "cat" => Some(Box::new(Cat::new(args))),
        "cp" => Some(Box::new(Cp::new(args, opts))),
        "rm" => Some(Box::new(Rm::new(args, opts))),
        "mv" => Some(Box::new(Mv::new(args))),
        "mkdir" => Some(Box::new(Mkdir::new(args, opts))),
        "export" => Some(Box::new(Export::new(args))),
        "exit" => {
            std::process::exit(0);
        }
        _ => None,
    }
}

fn execute_command_with_stdio(
    cmd_str: &str,
    args: &[String],
    stdin: Option<OwnedFd>,
    stdout: Option<OwnedFd>,
    stderr: Stdio,
    _is_first: bool,
    _is_last: bool,
    env: &mut ShellEnv,
    use_external: bool,
) -> Result<Child, ShellError> {
    if use_external {

        let env_result = ENV.lock();
        if let Ok(env_map) = env_result {
            if let Some(full_path) = env_map.get(cmd_str) {
                let mut command = ExternalCommand::new(full_path);
                command.args(args);
    
                if let Some(ref fd) = stdin {
                    let new_fd = dup(fd.as_raw_fd()).unwrap();
                    command.stdin(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
                } else {
                    command.stdin(Stdio::inherit());
                }
    
                if let Some(ref fd) = stdout {
                    let new_fd = dup(fd.as_raw_fd()).unwrap();
                    command.stdout(Stdio::from(unsafe { OwnedFd::from_raw_fd(new_fd) }));
                } else {
                    command.stdout(Stdio::inherit());
                }
    
                command.stderr(stderr);
    
                return command
                    .spawn()
                    .map_err(|e| ShellError::Exec(format!("Failed to spawn {}: {}", cmd_str, e)));
            }
        }
    }else {
        
    }
    Err(ShellError::Exec(format!(
        "External command not found: {}",
        cmd_str
    )))
}

fn should_use_external_for_pipeline(cmd: &str) -> bool {
    matches!(cmd, "ls" | "cat")
}
