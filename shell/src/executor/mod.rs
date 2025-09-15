mod merge_redirects_for_group;
mod exec_command;
mod exec_group;
mod exec_sequence;
mod exec_and;
mod exec_or;
mod exec_not;
mod exec_subshell;
mod exec_if;
mod exec_while;
mod exec_for;
mod exec_until;
mod parse_level;


use parse_level::parse_level;
mod exec_pipeline;
use nix::unistd::Pid;

use crate::envirement::ShellEnv;
use crate::error::ShellError;
use crate::parser::types::*;



pub struct Executor<'a> {
    pub env: &'a mut ShellEnv,
    pub job_group: Option<Pid>,
}

impl <'a>Executor <'a> {
    pub fn new(env : &'a mut ShellEnv) -> Self{
        return Self { env, job_group: None }
    }
}

impl<'a> Executor<'a> {
    pub fn execute_node(
        &mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        match node {
            AstNode::Command { .. } => self.exec_command(node, is_background),
            AstNode::Pipeline(_) => self.exec_pipeline(node, is_background, loop_depth),
            AstNode::Sequence(_) => { self.exec_sequence(node, is_background, loop_depth)},
            AstNode::Group {..} => { self.execute_group(node, is_background, loop_depth)}
            AstNode::Background(inner) => self.execute_node(inner, true, loop_depth),
            AstNode::And(left, right) => self.exec_and(left, right, is_background, loop_depth),
            AstNode::Or(left, right) => self.exec_or(left, right, is_background, loop_depth),
            AstNode::Not(inner) => self.exec_not(inner, is_background, loop_depth),
            AstNode::Subshell(inner) => self.exec_subshell(inner, is_background, loop_depth),
            AstNode::If { .. } => self.exec_if(node, is_background, loop_depth),
            AstNode::For { .. } => self.exec_for(node, is_background, loop_depth),
            AstNode::While { .. } => self.exec_while(node, is_background, loop_depth),
            AstNode::Until { .. } => self.exec_until(node, is_background, loop_depth),
            AstNode::Break(level) => {
                let n = parse_level(level, self.env, "break")?;
                let n = n.min(loop_depth);
                Err(ShellError::Break(n))
            }
            AstNode::Continue(level) => {
                let n = parse_level(level, self.env, "continue")?;
                let n = n.min(loop_depth);
                Err(ShellError::Continue(n))
            },

            AstNode::FunctionDef { ..} =>{
                return Ok(0);
            },
        }
    }
}
