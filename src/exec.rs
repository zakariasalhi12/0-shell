use crate::ShellCommand;
use crate::builtins::try_builtin;
use crate::commands::{
    cat::Cat, cd::Cd, cp::Cp, echo::Echo, export::Export, ls::Ls, mkdir::Mkdir, mv::Mv, pwd::Pwd,
    rm::Rm,
};
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::expansion::expand;
use crate::parser::types::*;
use std::process::Command as ExternalCommand;

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

            // 2. Handle assignments (TODO)
            if !assignments.is_empty() {
                // Placeholder: print assignments
                println!("[exec] Assignments: {:?}", assignments);
            }

            // 3. Handle redirects (TODO)
            if !redirects.is_empty() {
                println!("[exec] Redirects: {:?}", redirects);
            }

            // 4. Check for built-in
            let command = build_command(&cmd_str, arg_strs.clone(), opts);
            match command {
                Some(val) => {
                    let res = val.execute();
                    match res {
                        Ok(_) => Ok((0)),
                        Err(e) => {
                            println!("{e}\r");
                            Ok(1)
                        }
                    }
                }
                None => Ok(127),
            }

            // 5. Try to run as external command
            // let mut child = match ExternalCommand::new(&cmd_str).args(&arg_strs).spawn() {
            //     Ok(child) => child,
            //     Err(e) => {
            //         eprintln!("{}: command not found or failed to execute: {}", cmd_str, e);
            //         return Ok(127); // Common shell code for command not found
            //     }
            // };
            // let status = child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1);
            // Ok(status)
        }
        AstNode::Pipeline(nodes) => {
            // TODO: Execute each node in the pipeline, connect their input/output
            println!("[exec] Pipeline: {} commands", nodes.len());
            Ok(0)
        }
        AstNode::Sequence(nodes) => {
            // TODO: Execute each node in sequence
            println!("[exec] Sequence: {} commands", nodes.len());
            Ok(0)
        }
        AstNode::And(left, right) => {
            // TODO: Execute left, if success then right
            println!("[exec] And (&&)");
            Ok(0)
        }
        AstNode::Or(left, right) => {
            // TODO: Execute left, if fail then right
            println!("[exec] Or (||)");
            Ok(0)
        }
        AstNode::Not(node) => {
            // TODO: Execute node, invert status
            println!("[exec] Not (!)");
            Ok(0)
        }
        AstNode::Background(node) => {
            // TODO: Execute node in background (job control)
            println!("[exec] Background (&)");
            Ok(0)
        }
        AstNode::Subshell(node) => {
            // TODO: Execute node in a subshell (forked environment)
            println!("[exec] Subshell");
            Ok(0)
        }
        AstNode::Group {
            commands,
            redirects,
        } => {
            // TODO: Execute group of commands, handle redirects
            println!("[exec] Group: {} commands", commands.len());
            Ok(0)
        }
        AstNode::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // TODO: Execute condition, then then_branch or else_branch
            println!("[exec] If");
            Ok(0)
        }
        AstNode::While { condition, body } => {
            // TODO: Execute while loop
            println!("[exec] While");
            Ok(0)
        }
        AstNode::Until { condition, body } => {
            // TODO: Execute until loop
            println!("[exec] Until");
            Ok(0)
        }
        AstNode::For { var, values, body } => {
            // TODO: Execute for loop
            println!("[exec] For");
            Ok(0)
        }
        AstNode::Case { word, arms } => {
            // TODO: Execute case statement
            println!("[exec] Case");
            Ok(0)
        }
        AstNode::FunctionDef { name, body } => {
            // TODO: Register function in environment
            println!("[exec] FunctionDef");
            Ok(0)
        }
        AstNode::ArithmeticCommand(expr) => {
            // TODO: Evaluate arithmetic expression
            println!("[exec] ArithmeticCommand");
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
