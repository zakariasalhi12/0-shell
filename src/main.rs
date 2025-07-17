use shell::events_handler::*;
use termion::*;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
