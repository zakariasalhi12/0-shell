use cat::Cat;
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let cat = Cat::new(args);
    println!("Hello, world!");
    eprintln!("Hello, world!");

    io::stderr().write_all(b"Hello, world!\n").unwrap();

    cat.execute();
}
