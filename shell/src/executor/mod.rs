// use std::collections::HashMap;
// use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
// mod merge_redirects_for_group;
// use merge_redirects_for_group::merge_redirects;
// use crate::commands::jobs;
// use crate::envirement::ShellEnv;
// use crate::exec::CommandResult;
// use crate::lexer::types::{QuoteType, Word};
// use crate::parser::types::*;
// use crate::error::ShellError;
// use crate::executorr::spawn_commande::{spawn_command};
// use nix::unistd::Pid;
// use crate::features::jobs::{Job, JobStatus};
// use crate::exec::wait_for_single_process;
// use nix::unistd::pipe;
// use crate::commands::fals::False;
// use crate::commands::test::Test;
// use crate::commands::tru::True;
// // Modified exec.rs
// use crate::PathBuf;
// use crate::ShellCommand;
// use crate::commands::bg::Bg;
// use crate::commands::exit::Exit;
// use crate::commands::fg::Fg;
// use crate::commands::jobs::Jobs;
// use crate::commands::kill::Kill;
// use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
// use nix::unistd::setpgid;
// use nix::unistd::{getpgrp, tcsetpgrp};
// use std::fs::File;
// use std::os::fd::IntoRawFd;
// use crate::exec::wait_for_pipeline;

// // use crate::commands::{
// //     cd::Cd, cp::Cp, echo::Echo, export::Export, mkdir::Mkdir, mv::Mv, pwd::Pwd, rm::Rm, typ::Type,
// // };


// pub struct Executor<'a> {
//     pub env: &'a mut ShellEnv,
//     pub job_group: Option<Pid>,
// }

// impl<'a> Executor<'a> {
//     pub fn execute_node(
//         &mut self,
//         node: &AstNode,
//         is_background: bool,
//         loop_depth: usize,
//     ) -> Result<i32, ShellError> {
//         match node {
//             AstNode::Command { .. } => self.exec_command(node, is_background),
//             AstNode::Pipeline(_) => self.exec_pipeline(node, is_background, loop_depth),
//             AstNode::Sequence(_) | AstNode::Group { .. } => self.exec_sequence(node, is_background, loop_depth),
//             AstNode::Background(inner) => self.execute_node(inner, true, loop_depth),
//             AstNode::And(left, right) => self.exec_and(left, right, is_background, loop_depth),
//             AstNode::Or(left, right) => self.exec_or(left, right, is_background, loop_depth),
//             AstNode::Not(inner) => self.exec_not(inner, is_background, loop_depth),
//             AstNode::Subshell(inner) => self.exec_subshell(inner, is_background, loop_depth),
//             AstNode::If { .. } => self.exec_if(node, is_background, loop_depth),
//             AstNode::For { .. } | AstNode::While { .. } | AstNode::Until { .. } => {
//                 self.exec_loop(node, is_background, loop_depth)
//             }
//             // AstNode::Break(level) => {
//             //     let n = crate::exec::parse_level(level, self.env, "break")?;
//             //     let n = n.min(loop_depth);
//             //     Err(ShellError::Break(n))
//             // }
//             // AstNode::Continue(level) => {
//             //     let n = crate::exec::parse_level(level, self.env, "continue")?;
//             //     let n = n.min(loop_depth);
//             //     Err(ShellError::Continue(n))
//             // }
//             _ => Ok(0),
//         }
//     }

//     fn exec_command(&mut self, node: &AstNode, is_background: bool) -> Result<i32, ShellError> {
//         if let AstNode::Command { cmd, args, assignments, redirects } = node{
//             match spawn_command(cmd, args, assignments, redirects, self.env, None, &mut None)? {
//                 CommandResult::Child(pid) => {
//                     let merged = Word {
//                         parts: args.iter().flat_map(|w| w.parts.clone()).collect(),
//                         quote: QuoteType::None, // or however you want to handle quotes
//                     };
//                     if !is_background {
//                         let status = wait_for_single_process(
//                             pid,
//                             self.env,
//                             cmd.expand(self.env) + " " + &merged.expand(self.env),
//                         )?;
//                         self.env.set_last_status(status);
//                         return Ok(status)
//                     } else {
//                         // Add to jobs and don't wait

//                         let new_job = Job::new(
//                             pid,
//                             pid,
//                             self.env.jobs.size + 1,
//                             JobStatus::Running,
//                             cmd.expand(self.env) + " " + &merged.expand(self.env),
//                         );
//                         self.env.jobs.add_job(new_job.clone());
//                         self.env.jobs
//                             .get_job(new_job.pid.clone())
//                             .unwrap()
//                             .status
//                             .printStatus(self.env.jobs.get_job(new_job.pid.clone()).unwrap().clone());
//                         return Ok(0)
//                     }
//                 }
//                 CommandResult::Builtin(n) => return Ok(n),
//             }
//         }
//         unreachable!() 
//     }

//     fn exec_pipeline(&mut self, node: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         if let AstNode::Pipeline(nodes) = node {
//             if nodes.is_empty() {
//                 return Ok(0);
//             }

//             if nodes.len() == 1 {
//                 return self.execute_node(&nodes[0], is_background, loop_depth);
//             }

//             let mut prev_read: Option<OwnedFd> = None;
//             let mut pipeline_gid = Option::<Pid>::None; // This will store the pipeline's process group ID
//             let mut child_pids = Vec::<Pid>::new();

//             // Execute all commands in the pipeline concurrently
//             for (i, node) in nodes.iter().enumerate() {
//                 let is_last = i == nodes.len() - 1;
//                 let is_first = i == 0;

//                 if let AstNode::Command {
//                     cmd,
//                     args,
//                     assignments,
//                     redirects,
//                 } = node
//                 {
//                     let stdin = prev_read.take();

//                     // Create new pipe only if this is not the last command
//                     let (read_end, write_end) = if !is_last {
//                         let (read_fd, write_fd) = pipe().expect("pipe failed");
//                         (
//                             Some(unsafe { OwnedFd::from_raw_fd(read_fd.as_raw_fd()) }),
//                             Some(unsafe { OwnedFd::from_raw_fd(write_fd.as_raw_fd()) }),
//                         )
//                     } else {
//                         (None, None)
//                     };

//                     let fds_map = {
//                         let mut map: HashMap<u64, OwnedFd> = HashMap::new();
//                         if let Some(stdi) = stdin {
//                             map.insert(0, stdi);
//                         }
//                         if let Some(stdo) = write_end {
//                             map.insert(1, stdo);
//                         }
//                         Some(map)
//                     };

//                     // For the first command, we need to create a new process group
//                     // For subsequent commands, we add them to the existing group
//                     let mut current_gid = if is_first {
//                         None // Let spawn_command create a new group
//                     } else {
//                         pipeline_gid // Use the established group
//                     };

//                     // Spawn the command without waiting
//                     match spawn_command(
//                         cmd,
//                         args,
//                         assignments,
//                         redirects,
//                         self.env,
//                         fds_map.as_ref(),
//                         &mut current_gid,
//                     )? {
//                         CommandResult::Child(child_pid) => {
//                             child_pids.push(child_pid);

//                             // If this is the first command, its PID becomes the process group ID
//                             if is_first {
//                                 pipeline_gid = Some(child_pid);

//                                 // Make sure the first process becomes the group leader
//                                 // This should be done in spawn_command, but ensure it here
//                                 if let Err(e) = setpgid(child_pid, child_pid) {
//                                     eprintln!("Warning: Failed to set process group leader: {}", e);
//                                 }
//                             } else {
//                                 // Add subsequent processes to the pipeline's process group
//                                 if let Some(pgid) = pipeline_gid {
//                                     if let Err(e) = setpgid(child_pid, pgid) {
//                                         eprintln!("Warning: Failed to add process to group: {}", e);
//                                     }
//                                 }
//                             }
//                         }
//                         CommandResult::Builtin(n) => {
//                             // Builtin commands are executed immediately
//                             // In a real pipeline, builtins should also fork, but this is a simplification
//                         }
//                     }

//                     prev_read = read_end; // becomes stdin for next command
//                 } else {
//                     return Err(ShellError::Exec(
//                         "Pipeline can only contain commands".to_string(),
//                     ));
//                 }
//             }

//             // Now handle waiting based on whether it's background or not
//             if !child_pids.is_empty() {
//                 if let Some(pgid) = pipeline_gid {
//                     let pipeline_cmd = nodes
//                         .iter()
//                         .filter_map(|node| {
//                             if let AstNode::Command { cmd, .. } = node {
//                                 Some(cmd.expand(self.env))
//                             } else {
//                                 None
//                             }
//                         })
//                         .collect::<Vec<_>>()
//                         .join(" | ");

//                     if is_background {
//                         // Create job and add all processes to it
//                         let mut new_job = Job::new(
//                             pgid,
//                             pgid, // leader_pid is same as pgid for pipelines
//                             self.env.jobs.size + 1,
//                             JobStatus::Running,
//                             pipeline_cmd,
//                         );

//                         // Add all child processes to the job
//                         for (i, &pid) in child_pids.iter().enumerate() {
//                             let cmd_name = if let AstNode::Command { cmd, .. } = &nodes[i] {
//                                 cmd.expand(self.env)
//                             } else {
//                                 "unknown".to_string()
//                             };
//                             new_job.add_process(pid, cmd_name);
//                         }

//                         self.env.jobs.add_job(new_job);
//                         return Ok(0);
//                     } else {
//                         // For foreground pipelines, we don't add to jobs but still wait properly
//                         let status = wait_for_pipeline(pgid, child_pids, pipeline_cmd, self.env)?;
//                         self.env.set_last_status(status);
//                         return Ok(status);
//                     }
//                 }
//             }

//             Ok(0)
        
//         } else { Ok(0) }
//     }
//     fn execute_group(
//     commands: &[AstNode],
//     env: &mut ShellEnv,
//     is_background: bool,
//     loop_depth: usize,
//     group_redirects: &Option<Vec<Redirect>>
// ) -> Result<i32, ShellError> {
//     let mut last_status = 0;

//     for cmd in commands {
//         // Merge group redirects with command's own
//         let effective_redirects = merge_redirects(group_redirects, cmd.get_redirects());

//         last_status = execute_node_with_redirects(
//             cmd,
//             env,
//             is_background,
//             loop_depth,
//             &effective_redirects,
//         )?;
//     }

//     env.set_last_status(last_status);
//     Ok(last_status)
// }


//         fn exec_sequence(
//         &mut self,
//         nodes: &Vec<AstNode>,
//         is_background: bool,
//         loop_depth: usize,
//         redirects: Option<&Vec<Redirect>>,
//     ) -> Result<i32, ShellError> {
//         let mut last_status = 0;
//         for node in nodes {
//             last_status = self.execute_node(node, is_background, loop_depth, redirects)?;
//         }
//         self.env.set_last_status(last_status);
//         Ok(last_status)
//     }

//     fn exec_and(&mut self, left: &AstNode, right: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         let left_status = self.execute_node(left, is_background, loop_depth)?;
//         if left_status == 0 {
//             let right_status = self.execute_node(right, is_background, loop_depth)?;
//             self.env.set_last_status(right_status);
//             Ok(right_status)
//         } else {
//             self.env.set_last_status(left_status);
//             Ok(left_status)
//         }
//     }

//     fn exec_or(&mut self, left: &AstNode, right: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         let left_status = self.execute_node(left, is_background, loop_depth)?;
//         if left_status != 0 {
//             let right_status = self.execute_node(right, is_background, loop_depth)?;
//             self.env.set_last_status(right_status);
//             Ok(right_status)
//         } else {
//             self.env.set_last_status(left_status);
//             Ok(left_status)
//         }
//     }

//     fn exec_not(&mut self, node: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         let status = self.execute_node(node, is_background, loop_depth)?;
//         let inverted = if status == 0 { 1 } else { 0 };
//         self.env.set_last_status(inverted);
//         Ok(inverted)
//     }

//     fn exec_subshell(&mut self, node: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         let status = self.execute_node(node, is_background, loop_depth)?;
//         self.env.set_last_status(status);
//         Ok(status)
//     }

//     fn exec_if(&mut self, node: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         if let AstNode::If { condition, then_branch, elif, else_branch } = node {
//             let cond_status = self.execute_node(condition, is_background, loop_depth)?;
//             if cond_status == 0 {
//                 let status = self.execute_node(then_branch, is_background, loop_depth)?;
//                 self.env.set_last_status(status);
//                 return Ok(status);
//             }

//             let mut matched = false;
//             let mut status = cond_status;
//             for (elif_cond, elif_body) in elif {
//                 let elif_status = self.execute_node(elif_cond, is_background, loop_depth)?;
//                 if elif_status == 0 {
//                     status = self.execute_node(elif_body, is_background, loop_depth)?;
//                     matched = true;
//                     break;
//                 } else { status = elif_status; }
//             }

//             if !matched {
//                 if let Some(else_node) = else_branch {
//                     status = self.execute_node(else_node, is_background, loop_depth)?;
//                 }
//             }
//             self.env.set_last_status(status);
//             Ok(status)
//         } else { Ok(0) }
//     }

//     fn exec_loop(&mut self, node: &AstNode, is_background: bool, loop_depth: usize) -> Result<i32, ShellError> {
//         let new_depth = loop_depth + 1;
//         match node {
//             AstNode::For { var, values, body } => {
//                 let mut last_status = 0;
//                 for v in values {
//                     self.self.env.set_local_var(var, &v.expand(self.self.env));
//                     match self.execute_node(body, is_background, new_depth) {
//                         Err(ShellError::Break(mut n)) => { if n == 1 { break; } else { n -= 1; return Err(ShellError::Break(n)); } }
//                         Err(ShellError::Continue(mut n)) => { if n == 1 { continue; } else { n -= 1; return Err(ShellError::Continue(n)); } }
//                         Err(e) => return Err(e),
//                         Ok(s) => last_status = s,
//                     }
//                 }
//                 self.self.env.set_last_status(last_status);
//                 Ok(last_status)
//             }

//             AstNode::While { condition, body } => {
//                 let mut last_status = 0;
//                 loop {
//                     let cond_status = self.execute_node(condition, is_background, new_depth)?;
//                     if cond_status != 0 { break; }

//                     match self.execute_node(body, is_background, new_depth) {
//                         Err(ShellError::Break(mut n)) => { if n == 1 { break; } else { n -= 1; return Err(ShellError::Break(n)); } }
//                         Err(ShellError::Continue(mut n)) => { if n == 1 { continue; } else { n -= 1; return Err(ShellError::Continue(n)); } }
//                         Err(e) => return Err(e),
//                         Ok(s) => last_status = s,
//                     }
//                 }
//                 self.self.env.set_last_status(last_status);
//                 Ok(last_status)
//             }

//             AstNode::Until { condition, body } => {
//                 let mut last_status = 0;
//                 loop {
//                     let cond_status = self.execute_node(condition, is_background, new_depth)?;
//                     if cond_status == 0 { break; }

//                     match self.execute_node(body, is_background, new_depth) {
//                         Err(ShellError::Break(mut n)) => { if n == 1 { break; } else { n -= 1; return Err(ShellError::Break(n)); } }
//                         Err(ShellError::Continue(mut n)) => { if n == 1 { continue; } else { n -= 1; return Err(ShellError::Continue(n)); } }
//                         Err(e) => return Err(e),
//                         Ok(s) => last_status = s,
//                     }
//                 }
//                 self.self.env.set_last_status(last_status);
//                 Ok(last_status)
//             }

//             _ => Ok(0)
//         }
//     }
// }
