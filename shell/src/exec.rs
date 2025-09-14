use crate::commands::fals::False;
use crate::commands::test::Test;
use crate::commands::tru::True;
use crate::lexer::types::QuoteType;
use crate::lexer::types::Word;
// Modified exec.rs
use crate::PathBuf;
use crate::ShellCommand;
use crate::commands::bg::Bg;
use crate::commands::exit::Exit;
use crate::commands::fg::Fg;
use crate::commands::jobs::Jobs;
use crate::commands::kill::Kill;
use crate::executorr::spawn_commande::spawn_command;
use nix::sys::signal::{Signal, signal};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use nix::unistd::pipe;
use nix::unistd::setpgid;
use nix::unistd::{getpgrp, tcsetpgrp};
use std::collections::HashMap;
use std::fs::File;
use std::os::fd::IntoRawFd;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

use crate::commands::{
    cd::Cd, cp::Cp, echo::Echo, export::Export, mkdir::Mkdir, mv::Mv, pwd::Pwd, rm::Rm, typ::Type,
};
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::features::jobs;
use crate::features::jobs::JobStatus;
use crate::parser::types::*;

pub enum CommandResult {
    Child(Pid),
    Builtin(i32),
}

pub fn execute(ast: &AstNode, env: &mut ShellEnv) -> Result<i32, ShellError> {
    execute_with_background(ast, env, false, 0)
}

pub fn execute_with_background(
    ast: &AstNode,
    env: &mut ShellEnv,
    is_background: bool,
    loop_depth: usize,
) -> Result<i32, ShellError> {
    env.current_command = ast.to_text(env);
    match ast {
        AstNode::Command {
            cmd,
            args,
            assignments,
            redirects,
        } => {
            // For single commands, use the existing spawn logic
            match spawn_command(cmd, args, assignments, redirects, env, None, &mut None)? {
                CommandResult::Child(pid) => {
                    let merged = Word {
                        parts: args.iter().flat_map(|w| w.parts.clone()).collect(),
                        quote: QuoteType::None, // or however you want to handle quotes
                    };
                    if !is_background {
                        let status = wait_for_single_process(
                            pid,
                            env,
                            cmd.expand(env) + " " + &merged.expand(env),
                        )?;
                        env.set_last_status(status);
                        Ok(status)
                    } else {
                        // Add to jobs and don't wait

                        let new_job = jobs::Job::new(
                            pid,
                            pid,
                            env.jobs.size + 1,
                            jobs::JobStatus::Running,
                            cmd.expand(env) + " " + &merged.expand(env),
                        );
                        env.jobs.add_job(new_job.clone());
                        env.jobs
                            .get_job(new_job.pid.clone())
                            .unwrap()
                            .status
                            .printStatus(env.jobs.get_job(new_job.pid.clone()).unwrap().clone());
                        Ok(0)
                    }
                }
                CommandResult::Builtin(n) => Ok(n),
            }
        }
        AstNode::Pipeline(nodes) => {
            if nodes.is_empty() {
                return Ok(0);
            }

            if nodes.len() == 1 {
                return execute_with_background(&nodes[0], env, is_background, loop_depth);
            }

            let mut prev_read: Option<OwnedFd> = None;
            let mut pipeline_gid = Option::<Pid>::None; // This will store the pipeline's process group ID
            let mut child_pids = Vec::<Pid>::new();

            // Execute all commands in the pipeline concurrently
            for (i, node) in nodes.iter().enumerate() {
                let is_last = i == nodes.len() - 1;
                let is_first = i == 0;

                if let AstNode::Command {
                    cmd,
                    args,
                    assignments,
                    redirects,
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

                    // For the first command, we need to create a new process group
                    // For subsequent commands, we add them to the existing group
                    let mut current_gid = if is_first {
                        None // Let spawn_command create a new group
                    } else {
                        pipeline_gid // Use the established group
                    };

                    // Spawn the command without waiting
                    match spawn_command(
                        cmd,
                        args,
                        assignments,
                        redirects,
                        env,
                        fds_map.as_ref(),
                        &mut current_gid,
                    )? {
                        CommandResult::Child(child_pid) => {
                            child_pids.push(child_pid);

                            // If this is the first command, its PID becomes the process group ID
                            if is_first {
                                pipeline_gid = Some(child_pid);

                                // Make sure the first process becomes the group leader
                                // This should be done in spawn_command, but ensure it here
                                if let Err(e) = setpgid(child_pid, child_pid) {
                                    eprintln!("Warning: Failed to set process group leader: {}", e);
                                }
                            } else {
                                // Add subsequent processes to the pipeline's process group
                                if let Some(pgid) = pipeline_gid {
                                    if let Err(e) = setpgid(child_pid, pgid) {
                                        eprintln!("Warning: Failed to add process to group: {}", e);
                                    }
                                }
                            }
                        }
                        CommandResult::Builtin(n) => {
                            // Builtin commands are executed immediately
                            // In a real pipeline, builtins should also fork, but this is a simplification
                        }
                    }

                    prev_read = read_end; // becomes stdin for next command
                } else {
                    return Err(ShellError::Exec(
                        "Pipeline can only contain commands".to_string(),
                    ));
                }
            }

            // Now handle waiting based on whether it's background or not
            if !child_pids.is_empty() {
                if let Some(pgid) = pipeline_gid {
                    let pipeline_cmd = nodes
                        .iter()
                        .filter_map(|node| {
                            if let AstNode::Command { cmd, .. } = node {
                                Some(cmd.expand(env))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" | ");

                    if is_background {
                        // Create job and add all processes to it
                        let mut new_job = jobs::Job::new(
                            pgid,
                            pgid, // leader_pid is same as pgid for pipelines
                            env.jobs.size + 1,
                            jobs::JobStatus::Running,
                            pipeline_cmd,
                        );

                        // Add all child processes to the job
                        for (i, &pid) in child_pids.iter().enumerate() {
                            let cmd_name = if let AstNode::Command { cmd, .. } = &nodes[i] {
                                cmd.expand(env)
                            } else {
                                "unknown".to_string()
                            };
                            new_job.add_process(pid, cmd_name);
                        }

                        env.jobs.add_job(new_job);
                        return Ok(0);
                    } else {
                        // For foreground pipelines, we don't add to jobs but still wait properly
                        let status = wait_for_pipeline(pgid, child_pids, pipeline_cmd, env)?;
                        env.set_last_status(status);
                        return Ok(status);
                    }
                }
            }

            Ok(0)
        }

        AstNode::Background(node) => execute_with_background(node, env, true, loop_depth),

        AstNode::Sequence(nodes) => {
            let mut last_status = 0;
            for node in nodes {
                last_status = execute_with_background(node, env, is_background, loop_depth)?;
            }
            env.set_last_status(last_status);
            Ok(last_status)
        }

        AstNode::And(left, right) => {
            let left_status = execute_with_background(left, env, is_background, loop_depth)?;
            if left_status == 0 {
                let right_status = execute_with_background(right, env, is_background, loop_depth)?;
                env.set_last_status(right_status);
                Ok(right_status)
            } else {
                env.set_last_status(left_status);
                Ok(left_status)
            }
        }

        AstNode::Or(left, right) => {
            let left_status = execute_with_background(left, env, is_background, loop_depth)?;
            if left_status != 0 {
                let right_status = execute_with_background(right, env, is_background, loop_depth)?;
                env.set_last_status(right_status);
                Ok(right_status)
            } else {
                env.set_last_status(left_status);
                Ok(left_status)
            }
        }

        AstNode::Not(node) => {
            let status = execute_with_background(node, env, is_background, loop_depth)?;
            let inverted_status = if status == 0 { 1 } else { 0 };
            env.set_last_status(inverted_status);
            Ok(inverted_status)
        }

        AstNode::Subshell(node) => {
            let status = execute_with_background(node, env, is_background, loop_depth)?;
            env.set_last_status(status);
            Ok(status)
        }

        AstNode::Group {
            commands,
            redirects: _,
        } => {
            let mut last_status = 0;
            for command in commands {
                last_status = execute_with_background(command, env, is_background, loop_depth)?;
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
            let condition_status =
                execute_with_background(condition, env, is_background, loop_depth)?;
            if condition_status == 0 {
                let status = execute_with_background(then_branch, env, is_background, loop_depth)?;
                env.set_last_status(status);
                Ok(status)
            } else {
                let mut matched = false;
                let mut status = condition_status;

                for (elif_cond, elif_body) in elif.iter() {
                    let elif_cond_status =
                        execute_with_background(elif_cond, env, is_background, loop_depth)?;
                    if elif_cond_status == 0 {
                        status =
                            execute_with_background(elif_body, env, is_background, loop_depth)?;
                        matched = true;
                        break;
                    } else {
                        status = elif_cond_status;
                    }
                }

                if !matched {
                    if let Some(else_node) = else_branch {
                        status =
                            execute_with_background(else_node, env, is_background, loop_depth)?;
                    }
                }

                env.set_last_status(status);
                Ok(status)
            }
        }

        AstNode::For { var, values, body } => {
            let mut last_status = 0;
            let new_depth = loop_depth + 1; // entering a loop

            for v in values {
                env.set_local_var(&var, &v.expand(env));

                match execute_with_background(body, env, is_background, new_depth) {
                    Err(ShellError::Break(mut remaining)) => {
                        if remaining == 1 {
                            break;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Break(remaining));
                        }
                    }
                    Err(ShellError::Continue(mut remaining)) => {
                        if remaining == 1 {
                            continue;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Continue(remaining));
                        }
                    }
                    Err(e) => return Err(e),
                    Ok(status) => last_status = status,
                }
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }

        AstNode::While { condition, body } => {
            let mut last_status = 0;
            let new_depth = loop_depth + 1; // entering a loop

            loop {
                let condition_status =
                    execute_with_background(condition, env, is_background, new_depth)?;
                if condition_status != 0 {
                    break;
                }

                match execute_with_background(body, env, is_background, new_depth) {
                    Err(ShellError::Break(mut remaining)) => {
                        if remaining == 1 {
                            break;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Break(remaining));
                        }
                    }
                    Err(ShellError::Continue(mut remaining)) => {
                        if remaining == 1 {
                            continue;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Continue(remaining));
                        }
                    }
                    Err(e) => return Err(e),
                    Ok(status) => last_status = status,
                }
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }

        AstNode::Until { condition, body } => {
            let mut last_status = 0;
            let new_depth = loop_depth + 1;

            loop {
                let condition_status =
                    execute_with_background(condition, env, is_background, new_depth)?;
                if condition_status == 0 {
                    break;
                }

                match execute_with_background(body, env, is_background, new_depth) {
                    Err(ShellError::Break(mut remaining)) => {
                        if remaining == 1 {
                            break;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Break(remaining));
                        }
                    }
                    Err(ShellError::Continue(mut remaining)) => {
                        if remaining == 1 {
                            continue;
                        } else {
                            remaining -= 1;
                            return Err(ShellError::Continue(remaining));
                        }
                    }
                    Err(e) => return Err(e),
                    Ok(status) => last_status = status,
                }
            }

            env.set_last_status(last_status);
            Ok(last_status)
        }

        AstNode::Break(level_word) => {
            let n = parse_level(level_word, env, "break")?;
            let n = n.min(loop_depth);
            Err(ShellError::Break(n))
        }

        AstNode::Continue(level_word) => {
            let n = parse_level(level_word, env, "continue")?;
            let n = n.min(loop_depth);
            Err(ShellError::Continue(n))
        }

        _ => Ok(0),
    }
}

pub fn wait_for_single_process(pid: Pid, env: &mut ShellEnv, cmd: String) -> Result<i32, ShellError> {
    // Add job
    let new_job = jobs::Job::new(
        pid,
        pid,
        env.jobs.size + 1,
        jobs::JobStatus::Running,
        cmd, // You might want to pass the actual command
    );
    env.jobs.add_job(new_job);

    // Give terminal control
    let tty = File::open("/dev/tty").map_err(|e| ShellError::Io(e))?;
    let fd = tty.into_raw_fd();
    let shell_pgid = getpgrp();

    let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
        .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?;

    tcsetpgrp(fd, pid).map_err(|e| ShellError::Exec(format!("tcsetpgrp error: {}", e)))?;

    unsafe {
        signal(Signal::SIGTTOU, old)
            .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?
    };

    // Wait for process
    let exit_code = match waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
        Ok(wait_status) => match wait_status {
            WaitStatus::Exited(_, code) => {
                env.jobs.remove_job(pid);
                code
            }
            WaitStatus::Signaled(_, _, _) => {
                env.jobs.remove_job(pid);
                1
            }
            WaitStatus::Stopped(_, _) => {
                println!();
                env.jobs.update_job_status(pid, JobStatus::Stopped);
                1
            }
            _ => 1,
        },
        Err(_) => 1,
    };

    // Return terminal control to shell
    let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
        .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?;

    tcsetpgrp(fd, shell_pgid).ok();
    unsafe {
        signal(Signal::SIGTTOU, old)
            .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?
    };

    Ok(exit_code)
}

pub fn wait_for_pipeline(
    pgid: Pid,
    child_pids: Vec<Pid>,
    pipeline_cmd: String,
    env: &mut ShellEnv,
) -> Result<i32, ShellError> {
    // Add job for the entire pipeline
    let new_job = jobs::Job::new(
        pgid,
        pgid,
        env.jobs.size + 1,
        jobs::JobStatus::Running,
        pipeline_cmd,
    );
    env.jobs.add_job(new_job);

    // Give terminal control to the pipeline process group
    let tty = File::open("/dev/tty").map_err(|e| ShellError::Io(e))?;
    let fd = tty.into_raw_fd();
    let shell_pgid = getpgrp();

    let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
        .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?;

    tcsetpgrp(fd, pgid).map_err(|e| ShellError::Exec(format!("tcsetpgrp error: {}", e)))?;

    unsafe {
        signal(Signal::SIGTTOU, old)
            .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?
    };

    // Wait for all processes in the pipeline to complete
    let mut remaining_processes = child_pids.len();
    let mut pipeline_status = 0;

    while remaining_processes > 0 {
        match waitpid(
            Some(Pid::from_raw(-pgid.as_raw())), // Wait for any process in the group
            Some(WaitPidFlag::WUNTRACED),
        ) {
            Ok(wait_status) => match wait_status {
                WaitStatus::Exited(pid, code) => {
                    remaining_processes -= 1;
                    // Use exit code from the last command in pipeline
                    if child_pids.last() == Some(&pid) {
                        pipeline_status = code;
                    }
                }
                WaitStatus::Signaled(pid, _, _) => {
                    remaining_processes -= 1;
                    if child_pids.last() == Some(&pid) {
                        pipeline_status = 1;
                    }
                }
                WaitStatus::Stopped(_, _) => {
                    env.jobs.update_job_status(pgid, JobStatus::Stopped);
                    println!();
                    pipeline_status = 1;
                    break;
                }
                _ => {}
            },
            Err(_) => {
                remaining_processes = 0;
                pipeline_status = 1;
            }
        }
    }

    if remaining_processes == 0 {
        env.jobs.remove_job(pgid);
    }

    // Return terminal control to shell
    let old = unsafe { signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn) }
        .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?;

    tcsetpgrp(fd, shell_pgid).ok();
    unsafe {
        signal(Signal::SIGTTOU, old)
            .map_err(|e| ShellError::Exec(format!("Signal error: {}", e)))?
    };

    Ok(pipeline_status)
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
        "test" => Some(Box::new(Test::new(args, false))),
        "[" => Some(Box::new(Test::new(args, true))),
        "true" => Some(Box::new(True::new(args))),
        "false" => Some(Box::new(False::new(args))),
        _ => None,
    }
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
        | "fg" | "jobs" | "kill" | "bg" | "test" | "[" | "true" | "false" => CommandType::Builtin,
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

fn parse_level(word: &Option<Word>, env: &ShellEnv, cmd: &str) -> Result<usize, ShellError> {
    let level_str = match word {
        Some(w) => w.expand(env),
        None => return Ok(1),
    };

    match level_str.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => {
            return Err(ShellError::Push(format!(
                "{}: {}: numeric argument required",
                cmd, level_str
            )));
        }
    }
}
