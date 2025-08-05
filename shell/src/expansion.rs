use crate::{envirement::ShellEnv, lexer::{types::{QuoteType, Word}}};

    pub fn expand_and_split(word: &Word, env: &ShellEnv) -> Vec<String> {
    let expanded = word.expand(env);
    if word.quote == QuoteType::None {
        expanded.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        vec![expanded]
    }

}