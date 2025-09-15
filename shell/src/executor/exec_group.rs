
use crate::executor::merge_redirects_for_group::merge_command_redirects_with_group;
use crate::{
    error::ShellError,
    executor::Executor,
    types::AstNode,
};

impl<'a> Executor<'a> {
   
    pub fn execute_group(&mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,) -> Result<i32, ShellError> {

        if let AstNode::Group { commands, redirects: group_redirects } = node{
              let mut last_status = 0;

        for cmd in commands {
            if let AstNode::Command { cmd, args, assignments, redirects: command_redirects } = cmd{
                let effective_redirects = merge_command_redirects_with_group(group_redirects.to_owned(), command_redirects.to_owned());
                let new_cmd = AstNode::Command { cmd: cmd.clone(), args: args.clone(), assignments: assignments.clone(), redirects: effective_redirects };
                last_status =  self.execute_node(
                    &new_cmd,
                    is_background,
                    loop_depth
                )?;
            }
            self.env.set_last_status(last_status);

        }
        return Ok(last_status)
        }
      
        unreachable!();
    }

}