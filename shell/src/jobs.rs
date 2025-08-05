use std::time::SystemTime;
use crate::parser::types::AstNode;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: usize,                        // shell job ID, e.g., 1 for %1
    pub pgid: Option<u32>,               // process group ID
    pub pids: Vec<u32>,                  // child process IDs
    pub command: String,                 // original user input
    pub status: JobStatus,               // current job status
    pub background: bool,                // was it run with '&'?
    pub created: SystemTime,             // for ordering / cleanup
}

pub fn launch_job(_: &AstNode, _: bool) -> Job { 
    todo!("");
 }
pub fn wait_for_job(_: &Job) -> i32 { 
    todo!("");
 }
pub fn list_jobs() { 
    todo!("");
 }
