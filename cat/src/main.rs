use cat::Cat;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cat = Cat::new(args[1..].to_vec());

    match cat.execute() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}
