use ls::Ls;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cat = Ls::new(args[1..].to_vec());
    
    cat.execute();
}
