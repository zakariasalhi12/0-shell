use crate::{envirement::ShellEnv, lexer::{self, types::{QuoteType, Word, WordPart}}, Parser};

pub fn expand_and_split(word: &Word, env: &ShellEnv) -> Vec<String> {
    let expanded = word.expand(env);
    expanded.split_whitespace().map(|s| s.to_string()).collect()
}