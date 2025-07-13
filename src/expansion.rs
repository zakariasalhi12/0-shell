use crate::env::ShellEnv;


pub fn expand(input: &str, env: &ShellEnv) -> String {
    // handle $VAR, ${VAR:-default}, $((1+2)), etc.
    return  String::from("");
}
