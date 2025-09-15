use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_and(
        &mut self,
        left: &AstNode,
        right: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        let left_status = self.execute_node(left, is_background, loop_depth)?;
        if left_status == 0 {
            let right_status = self.execute_node(right, is_background, loop_depth)?;
            self.env.set_last_status(right_status);
            return Ok(right_status);
        } else {
            self.env.set_last_status(left_status);
            return Ok(left_status);
        }
    }
}
