use shell::shell::*;

fn main() {
    let mut shell = match Shell::new() {
        Ok(val) => val,
        Err(_) => return,
    };
    shell.run();
}
