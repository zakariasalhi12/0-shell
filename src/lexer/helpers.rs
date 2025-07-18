use std::{char, iter::Peekable};

pub fn is_var_char(c : char) -> bool{
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn extract_until_matching<I : Iterator<Item = char>>(chars : Peekable<I>,start : &str, end : &str) -> String{
    let acc = String::new();
        
    acc
}