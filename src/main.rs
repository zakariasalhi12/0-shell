use shell::*;

fn main() {
    let f = std::io::stdin();
    loop {
        let mut input = String::from("");
        f.read_line(&mut input);
        let command = Cmd::new(input);
        command.parse_exec();
    }
}
