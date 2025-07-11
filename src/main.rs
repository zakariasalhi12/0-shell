mod config;
mod parser;
mod shell_handler;
pub mod executer;
pub use parser::*;

use shell_handler::*;


fn main() {
    let mut shell = Shell::new();
    shell.run();
}





// fn main() {
//     let stdin = stdin();
//     let mut stdout = stdout().into_raw_mode().unwrap();
//     stdout.flush().unwrap();

//     for c in stdin.events() {
//         let evt = c.unwrap();
//         match evt {
//             Event::Key(Key::Up) => break,
//             }
//             _ => {}
//         }
//         stdout.flush().unwrap();
//     }
// }