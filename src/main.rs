use shell::events_handler::*;

fn main() {
    let mut shell = match Shell::new() {
        Ok(val) => val,
        Err(e) => return,
    };
    shell.run();
}
