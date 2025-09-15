use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_subshell(
        &mut self,
        node : &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
          let status = self.execute_node(node, is_background, loop_depth)?;
            self.env.set_last_status(status);
            Ok(status)
    }
}
