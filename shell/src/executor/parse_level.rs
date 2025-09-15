use crate::{envirement::ShellEnv, error::ShellError, lexer::types::Word};

pub fn parse_level(word: &Option<Word>, env: &ShellEnv, cmd: &str) -> Result<usize, ShellError> {
    let level_str = match word {
        Some(w) => w.expand(env),
        None => return Ok(1),
    };

    match level_str.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => {
            return Err(ShellError::Push(format!(
                "{}: {}: numeric argument required",
                cmd, level_str
            )));
        }
    }
}
