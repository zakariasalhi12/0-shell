use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_while(
        &mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        if let AstNode::Until { condition, body } = node {
            let mut last_status = 0;
            let new_depth = loop_depth + 1;

            loop {
                let condition_status =
                    self.execute_node(condition, is_background, new_depth)?;
                if condition_status == 0 {
                    break;
                }

                match self.execute_node(body, is_background, new_depth) {
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

            self.env.set_last_status(last_status);
            return Ok(last_status)
        }
        unreachable!()
    }
}