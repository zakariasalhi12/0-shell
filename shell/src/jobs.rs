// use std::time::SystemTime;
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum JobStatus {
//     Running,
//     Stopped,
//     Terminated,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct Job {
//     pub id: usize,                        // shell job ID, e.g., 1 for %1
//     pub pgid: Option<u32>,               // process group ID
//     pub pids: Vec<u32>,                  // child process IDs
//     pub command: String,                 // original user input
//     pub status: JobStatus,               // current job status
//     pub background: bool,                // was it run with '&'?
// }