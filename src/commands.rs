use std::io::Write;
pub fn echo(args: Vec<&str>) {
    if args.len() == 0 {
        println!("");
        return;
    }
    args.iter().enumerate().for_each(|(i, c)| {
        if i != args.len() - 1{
            print!("{c} ");
            std::io::stdout().flush();
        }else {
             println!("{c}");
        }
        
    });
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
