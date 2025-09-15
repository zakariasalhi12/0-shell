use crate::types::{Redirect, RedirectOp};

pub fn merge_command_redirects_with_group(
    group_redirects: Vec<Redirect>,
    cmd_redirects: Vec<Redirect>,
) -> Vec<Redirect> {
    use std::collections::HashMap;

    let mut merged: HashMap<u64, Redirect> = HashMap::new();

    let default_fd = |r: &Redirect| match r.kind {
        RedirectOp::Read | RedirectOp::HereDoc => 0,   // stdin
        RedirectOp::Write | RedirectOp::Append | RedirectOp::ReadWrite => 1, // stdout
    };

    // Apply group redirects first
    if group_redirects.len() > 0 {
        for r in group_redirects {
            let fd = r.fd.unwrap_or_else(|| default_fd(&r));
            merged.insert(fd, r.clone());
        }
    }

    // Override with command redirects
    if cmd_redirects.len() > 0 {
        for r in cmd_redirects {
            let fd = r.fd.unwrap_or_else(|| default_fd(&r));
            merged.insert(fd, r.clone());
        }
    }

    merged.into_values().collect()
}