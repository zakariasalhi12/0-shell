use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_not(
        &mut self,
        node : &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
          let status = self.execute_node(node, is_background, loop_depth)?;
            let inverted_status = if status == 0 { 1 } else { 0 };
            self.env.set_last_status(inverted_status);
            Ok(inverted_status)
    }
}
