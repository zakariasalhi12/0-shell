// use crate::types::{Redirect, RedirectOp};

// pub fn merge_redirects(
//     group_redirects: &Option<Vec<Redirect>>,
//     cmd_redirects: &Option<Vec<Redirect>>,
// ) -> Vec<Redirect> {
//     use std::collections::HashMap;

//     let mut merged: HashMap<u64, Redirect> = HashMap::new();

//     let default_fd = |r: &Redirect| match r.kind {
//         RedirectOp::Read | RedirectOp::HereDoc => 0,   // stdin
//         RedirectOp::Write | RedirectOp::Append | RedirectOp::ReadWrite => 1, // stdout
//     };

//     // Apply group redirects first
//     if let Some(gr) = group_redirects {
//         for r in gr {
//             let fd = r.fd.unwrap_or_else(|| default_fd(r));
//             merged.insert(fd, r.clone());
//         }
//     }

//     // Override with command redirects
//     if let Some(cr) = cmd_redirects {
//         for r in cr {
//             let fd = r.fd.unwrap_or_else(|| default_fd(r));
//             merged.insert(fd, r.clone());
//         }
//     }

//     merged.into_values().collect()
// }