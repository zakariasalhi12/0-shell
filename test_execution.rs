use crate::builtins::try_builtin;
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
use crate::ShellCommand;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::process::{Child, Command as ExternalCommand, Stdio};

// Add this to your imports or type definitions
#[derive(Debug, Clone)]
pub enum RedirectKind {
    Read,  //
    Write, // >
    Append, // >>
           // Add other kinds as needed
}

#[derive(Debug, Clone)]
pub struct Redirect {
    pub fd: Option<u32>,
    pub kind: RedirectKind,
    pub target: Word,
}

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

// Helper function to handle redirections
fn setup_redirections(
    redirects: &[Redirect],
    env: &ShellEnv,
) -> Result<(Stdio, Stdio, Stdio), ShellError> {
    let mut stdin_stdio = Stdio::inherit();
    let mut stdout_stdio = Stdio::inherit();
    let mut stderr_stdio = Stdio::inherit();

    for redirect in redirects {
        // Convert target Word to string
        let target_file = word_to_string(&redirect.target, env);

        // Determine which fd to redirect based on the redirect structure
        // You'll need to adjust this based on your actual Redirect struct
        match redirect {
            Redirect {
                fd: None,
                kind: RedirectKind::Write,
                ..
            } => {
                // stdout redirect ">"
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        stdout_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for writing: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            Redirect {
                fd: Some(1),
                kind: RedirectKind::Write,
                ..
            } => {
                // explicit stdout redirect "1>"
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        stdout_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for writing: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            Redirect {
                fd: None,
                kind: RedirectKind::Append,
                ..
            } => {
                // stdout append ">>"
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        stdout_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for appending: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            Redirect {
                fd: Some(2),
                kind: RedirectKind::Write,
                ..
            } => {
                // stderr redirect "2>"
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        stderr_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for stderr: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            Redirect {
                fd: Some(2),
                kind: RedirectKind::Append,
                ..
            } => {
                // stderr append "2>>"
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&target_file)
                {
                    Ok(file) => {
                        stderr_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for stderr append: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            Redirect {
                fd: None,
                kind: RedirectKind::Read,
                ..
            } => {
                // stdin redirect "<"
                match OpenOptions::new().read(true).open(&target_file) {
                    Ok(file) => {
                        stdin_stdio = Stdio::from(file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open {} for reading: {}", target_file, e);
                        return Err(ShellError::Exec(format!("redirect failed: {}", e)));
                    }
                }
            }
            _ => {
                eprintln!("Unsupported redirection: {:?}", redirect);
            }
        }
    }

    Ok((stdin_stdio, stdout_stdio, stderr_stdio))
}

// Check if a command should be executed as external (pipeable) even if it has a builtin
fn should_use_external_for_pipeline(cmd: &str) -> bool {
    matches!(cmd, "ls" | "cat" | "grep" | "sort" | "head" | "tail" | "wc")
}

// Execute a single command with specific stdio
fn execute_command_with_stdio(
    cmd_str: &str,
    args: &[String],
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
    env: &mut ShellEnv,
    use_external: bool,
) -> Result<Child, ShellError> {
    if use_external {
        // Force external execution for pipeline compatibility
        let env_result = ENV.lock();
        if let Ok(env_map) = env_result {
            if let Some(full_path) = env_map.get(cmd_str) {
                let child = ExternalCommand::new(full_path)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()
                    .map_err(|e| ShellError::Exec(format!("Failed to spawn {}: {}", cmd_str, e)))?;
                return Ok(child);
            }
        }
        return Err(ShellError::Exec(format!(
            "External command not found: {}",
            cmd_str
        )));
    } else {
        // Try builtin first
        let opts: Vec<String> = args
            .iter()
            .filter(|v| v.starts_with('-'))
            .cloned()
            .collect();

        let arg_strs: Vec<String> = args
            .iter()
            .filter(|v| !v.starts_with('-'))
            .cloned()
            .collect();

        if let Some(command) = build_command(&cmd_str.to_string(), arg_strs, opts) {
            // For builtins in pipeline, we need to handle them differently
            // This is a simplified approach - you might need to modify your builtin commands
            // to accept stdin/stdout parameters
            match command.execute() {
                Ok(_) => {
                    // For builtins, we can't return a Child, so this is a limitation
                    // You'd need to refactor builtins to work with pipes
                    return Err(ShellError::Exec(
                        "Builtin commands don't support pipes yet".to_string(),
                    ));
                }
                Err(e) => return Err(ShellError::Exec(format!("Builtin failed: {}", e))),
            }
        }

        // Fall back to external
        let env_result = ENV.lock();
        if let Ok(env_map) = env_result {
            if let Some(full_path) = env_map.get(cmd_str) {
                let child = ExternalCommand::new(full_path)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()
                    .map_err(|e| ShellError::Exec(format!("Failed to spawn {}: {}", cmd_str, e)))?;
                return Ok(child);
            }
        }

        Err(ShellError::Exec(format!("Command not found: {}", cmd_str)))
    }
}

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            // 1. Expand command and args
            let cmd_str = word_to_string(cmd, env);
            let all_args: Vec<String> = args.iter().map(|w| word_to_string(w, env)).collect();

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

            // 3. Handle redirects
            let (stdin_stdio, stdout_stdio, stderr_stdio) = if !redirects.is_empty() {
                setup_redirections(redirects, env)?
            } else {
                (Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
            };

            // 4. Check for built-in or function
            if !cmd_str.is_empty() {
                // Check if a function in environment functions
                if let Some(func) = env.get_func(&cmd_str) {
                    let body = func.clone();
                    let status = execute(&body, env)?;
                    env.set_last_status(status);
                    return Ok(status);
                }

                // For non-pipeline commands, try builtins first
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
                        if let Ok(env_map) = env_result {
                            if let Some(full_path) = env_map.get(&cmd_str) {
                                let mut child = ExternalCommand::new(full_path)
                                    .args(&all_args)
                                    .stdin(stdin_stdio)
                                    .stdout(stdout_stdio)
                                    .stderr(stderr_stdio)
                                    .spawn()
                                    .map_err(|e| {
                                        ShellError::Exec(format!(
                                            "Failed to execute {}: {}",
                                            cmd_str, e
                                        ))
                                    })?;

                                let status =
                                    child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
                                env.set_last_status(status);
                                Ok(status)
                            } else {
                                eprintln!("{}: command not found", cmd_str);
                                env.set_last_status(127);
                                Ok(127)
                            }
                        } else {
                            Err(ShellError::Exec(cmd_str))
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
                    let cmd_str = word_to_string(cmd, env);
                    let all_args: Vec<String> =
                        args.iter().map(|w| word_to_string(w, env)).collect();

                    // Setup stdio for this command in the pipeline
                    let stdin = if is_first {
                        Stdio::inherit()
                    } else if let Some(prev_out) = prev_stdout.take() {
                        Stdio::from(prev_out)
                    } else {
                        Stdio::null()
                    };

                    let stdout = if is_last {
                        // Handle redirections for the last command
                        if !redirects.is_empty() {
                            let (_, stdout_stdio, _) = setup_redirections(redirects, env)?;
                            stdout_stdio
                        } else {
                            Stdio::inherit()
                        }
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
            let status = execute(node, env)?;
            env.set_last_status(status);
            Ok(status)
        }
        AstNode::Subshell(node) => {
            // Execute node in a subshell (basic implementation)
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

            // Handle redirects
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
            let condition_status = execute(condition, env)?;

            if condition_status == 0 {
                let status = execute(then_branch, env)?;
                env.set_last_status(status);
                Ok(status)
            } else {
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
            let mut last_status = 0;

            loop {
                let condition_status = execute(condition, env)?;
                if condition_status != 0 {
                    break;
                }

                last_status = execute(body, env)?;
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::Until { condition, body } => {
            let mut last_status = 0;

            loop {
                let condition_status = execute(condition, env)?;
                if condition_status == 0 {
                    break;
                }

                last_status = execute(body, env)?;
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::For { var, values, body } => {
            let mut last_status = 0;

            for value in values {
                env.set_var(var, value);
                last_status = execute(body, env)?;
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }
        AstNode::Case { word, arms } => {
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
            let func_name = word_to_string(name, env);
            env.set_func(func_name, body.as_ref().clone());
            env.set_last_status(0);
            Ok(0)
        }
        AstNode::ArithmeticCommand(expr) => {
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
        "ls" => Some(Box::new(Ls::new(args))),
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
