use std::io::Write;
pub fn echo(args: Vec<&str>) {
    if args.len() == 0 {println!("");return}
    args.iter().for_each(|c| {print!("{c} ");std::io::stdout().flush();});
}
pub fn cd() {}
pub fn ls() {}
pub fn pwd() {}
pub fn cat() {}
pub fn cp() {}
pub fn rm() {}
pub fn mv() {}
pub fn mkdir() {}
pub fn exit() {}
