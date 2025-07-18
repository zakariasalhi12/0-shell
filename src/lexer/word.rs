use std::iter::Peekable;

use crate::{error::ShellError, lexer::{helpers::is_var_char, types::WordPart}};

pub fn parse_word<I : Iterator<Item = char>>(chars : &mut Peekable<I>) -> Result<Vec<WordPart>, ShellError>{
    let mut word_parts: Vec<WordPart> = vec![];

    while let Some(c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '&' | '|' | ';' => break,
            '$' => {
               chars.next();
               match chars.peek() {
                   Some('(') =>{
                    
                   },
                   _=>{
                    todo!()
                   }
               }
            },
            _ =>{
                todo!()
            }
        }
    }
    
    Ok(vec![])
}