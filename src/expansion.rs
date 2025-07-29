use crate::envirement::ShellEnv;

pub fn expand(input: &str, env: &ShellEnv) -> String {
    // handle $VAR, ${VAR:-default}, $((1+2)), etc.
    return  String::from("");
}
