use core::error;

use shell::events_handler::*;
use termion::*;

fn main() {
    let mut shell = match Shell::new() {
        Ok(val) => val,
        Err(e) => return,
    };

    shell.run();
}
