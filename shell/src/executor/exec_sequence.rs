use crate::{error::ShellError, executor::Executor, types::AstNode};

impl<'a> Executor<'a> {
    pub fn exec_sequence(
        &mut self,
        node: &AstNode,
        is_background: bool,
        loop_depth: usize,
    ) -> Result<i32, ShellError> {
        if let AstNode::Sequence(nodes) = node {
            let mut last_status = 0;
            for node in nodes {
                last_status = self.execute_node(node, is_background, loop_depth)?;
            }
            self.env.set_last_status(last_status);
            return Ok(last_status);
        }
        unreachable!()
    }
}
