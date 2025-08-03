use crate::ShellCommand;
use crate::builtins::try_builtin;
use crate::commands::{
    cat::Cat, cd::Cd, cp::Cp, echo::Echo, export::Export, ls::Ls, mkdir::Mkdir, mv::Mv, pwd::Pwd,
    rm::Rm,
};
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::expansion::expand_and_split;
use crate::lexer::types::Word;
use crate::parser::types::*;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::process::Child;
use std::process::Command as ExternalCommand;
use std::process::Stdio;

// fn word_to_string(word: &crate::lexer::types::Word, env: &ShellEnv) -> String {
//     // Expand and join all parts (for now, just join literals)
//     let mut result = String::new();
//     for part in &word.parts {
//         match part {
//             crate::lexer::types::WordPart::Literal(s) => result.push_str(s),
//             crate::lexer::types::WordPart::VariableSubstitution(var) => {
//                 if let Some(val) = env.get_var(var) {
//                     result.push_str(val);
//                 }
//             }
//             // TODO: handle ArithmeticSubstitution, CommandSubstitution
//             _ => {}
//         }
//     }
//     result
// }

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    let mut scoped_env: HashMap<String, String> = HashMap::new();
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            // println!("{}", ast);
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
                    scoped_env.insert(ass.0, ass.1.expand(&env));
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
                        let res = val.execute(env);
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
                            // Get the full path from your environment map
                            if let Some(full_path) = env.get(&cmd_str) {
                                println!("Found command at: {}", full_path);

                                // Use the full path instead of just the command name
                                let mut child =
                                    match ExternalCommand::new(full_path.clone()) // Use full_path here
                                        .args(&all_args)
                                        // .envs()
                                        .stdin(Stdio::inherit())
                                        .stdout(Stdio::inherit())
                                        .stderr(Stdio::inherit())
                                        .spawn()
                                    {
                                        Ok(child) => child,
                                        Err(e) => {
                                            eprintln!(
                                                "{}: command failed to execute: {}",
                                                full_path, e);
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
                // Single command, not really a pipeline
                return execute(&nodes[0], env);
            }

            let mut children: Vec<Child> = Vec::new();
            let mut prev_stdout: Option<std::process::ChildStdout> = None;

            for (i, node) in nodes.iter().enumerate() {
                let is_first = i == 0;
                let is_last = i == nodes.len() - 1;

                // Extract command info from the node
                if let AstNode::Command {
                    cmd,
                    args,
                    redirects,
                    ..
                } = node
                {
                    let cmd_str: String = cmd.expand(&env);
                    let all_args: Vec<String> = args.iter().map(|w| w.expand(env)).collect();

                    // Setup stdio for this command in the pipeline
                    let stdin = if is_first {
                        Stdio::inherit()
                    } else {
                        match prev_stdout.take() {
                            Some(stdout) => Stdio::from(stdout),
                            None => {
                                return Err(ShellError::Exec(
                                    "Pipeline broken: missing stdout".to_string(),
                                ));
                            }
                        }
                    };

                    let stdout = if is_last {
                        Stdio::inherit()
                    } else {
                        Stdio::piped()
                    };

                    let stderr = Stdio::inherit();
                    // Force external execution for pipeline-compatible commands
                    let use_external = should_use_external_for_pipeline(&cmd_str);
                    let mut child = execute_command_with_stdio(
                        &cmd_str,
                        &all_args,
                        stdin,
                        stdout,
                        stderr,
                        env,
                        use_external,
                    )?;
                    if let Some(ref stdin) = child.stdout {
                        println!("{i}");
                        let fd = stdin.as_raw_fd();
                        println!("stdin: {}", detect_fd_type(fd));
                    }
                    // println!("{:?} {:?} {:?}", &stdin, stdout, stderr);
                    // Capture stdout for the next command if not the last
                    if !is_last {
                        prev_stdout = child.stdout.take();
                    }

                    children.push(child);
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }

            // Wait for all children and get the status of the last one
            let mut last_status = 0;
            for mut child in children {
                let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
                last_status = status; // In bash, pipeline status is the status of the last command
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
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
    env: &mut ShellEnv,
    use_external: bool,
) -> Result<Child, ShellError> {
        if let Some(full_path) = env.get(cmd_str) {
            let child = match ExternalCommand::new(full_path)
                .args(args)
                .stdin(stdin)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .map_err(|e| ShellError::Exec(format!("Failed to spawn {}: {}", cmd_str, e)))
            {
                Ok(val) => val,
                Err(e) => {
                    println!("{:?}", e);
                    return Err(e);
                }
            };
            return Ok(child);
        }
    return Err(ShellError::Exec(format!(
        "External command not found: {}",
        cmd_str
    )));
}

fn should_use_external_for_pipeline(cmd: &str) -> bool {
    matches!(cmd, "ls" | "cat" | "grep" | "sort" | "head" | "tail" | "wc")
}

//////////////////////////////////////
use libc::{S_IFCHR, S_IFIFO, S_IFMT, fstat, isatty, stat};
use std::os::unix::io::AsRawFd;
use std::process::Command;

fn detect_fd_type(fd: i32) -> &'static str {
    let mut statbuf: stat = unsafe { std::mem::zeroed() };
    if unsafe { fstat(fd, &mut statbuf) } != 0 {
        return "Unknown";
    }
    let file_type = statbuf.st_mode & S_IFMT;
    match file_type {
        S_IFIFO => "Pipe",
        S_IFCHR => {
            if unsafe { isatty(fd) } == 1 {
                "TTY"
            } else {
                "Char Device (non-TTY)"
            }
        }
        _ => "Other",
    }
}
