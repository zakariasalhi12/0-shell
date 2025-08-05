use ls::Ls;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ls = Ls::new(args[1..].to_vec());
    match ls.execute() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
