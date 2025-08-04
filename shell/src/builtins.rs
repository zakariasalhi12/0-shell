use crate::envirement::ShellEnv;

pub fn try_builtin(args: &[String], env: &mut ShellEnv) -> Option<i32> {
    match args[0].as_str() {
        _ => None,
    }
}
