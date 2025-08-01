use cat::Cat;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cat = Cat::new(args);
    // println!("Hello, world!");
    cat.execute();
}
