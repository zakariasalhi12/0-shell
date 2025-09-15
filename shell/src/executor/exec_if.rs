use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_if(
        &mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        if let AstNode::If {
            condition,
            then_branch,
            elif,
            else_branch,
        } = node
        {
            let condition_status = self.execute_node(condition, is_background, loop_depth)?;
            if condition_status == 0 {
                let status = self.execute_node(then_branch, is_background, loop_depth)?;
                self.env.set_last_status(status);
                return Ok(status);
            } else {
                let mut matched = false;
                let mut status = condition_status;

                for (elif_cond, elif_body) in elif.iter() {
                    let elif_cond_status =
                        self.execute_node(elif_cond, is_background, loop_depth)?;
                    if elif_cond_status == 0 {
                        status = self.execute_node(elif_body, is_background, loop_depth)?;
                        matched = true;
                        break;
                    } else {
                        status = elif_cond_status;
                    }
                }

                if !matched {
                    if let Some(else_node) = else_branch {
                        status = self.execute_node(else_node, is_background, loop_depth)?;
                    }
                }

                self.env.set_last_status(status);
                return Ok(status);
            }
        }
        unreachable!()
    }
}
