use std::io::Write;
use std::*;
pub fn echo(args: Vec<String>) {
    if args.len() == 0 {
        println!("");
        return;
    }
    args.iter().enumerate().for_each(|(i, c)| {
        if i != args.len() - 1 {
            print!("{c} ");
            std::io::stdout().flush();
        } else {
            println!("{c}");
        }
    });
}
pub fn mkdir(args: Vec<String>) {
    if args.len() == 0 {
        println!("mkdir: missing operand\nTry 'mkdir --help' for more information.");
        return;
    }
    if args[0] == "--help" {
        println!("use only mkdir if you wish to make a dir without options !");
        return;
    }
}
pub fn pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.to_string_lossy());
        return;
    }
    println!("Error!");
}
pub fn cd() {}
pub fn ls() {}
pub fn cat() {}
pub fn cp() {}
pub fn rm() {}
pub fn mv() {}
pub fn exit() {}
