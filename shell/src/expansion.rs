use crate::{envirement::ShellEnv, lexer::{self, types::{QuoteType, Word, WordPart}}, Parser};

pub fn expand_and_split(word: &Word, env: &ShellEnv) -> Vec<String> {
    let expanded = word.expand(env);
    if word.quote == QuoteType::None {
        expanded.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        vec![expanded]
    }
}