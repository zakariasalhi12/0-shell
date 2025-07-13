use crate::error::ShellError;

#[derive(Debug, Clone)]
pub enum Token {
    Word(String),                 // any word or argument
    Assignment(String, String),   // FOO=bar
    AndIf, OrIf,                  // &&, ||
    Pipe, Amp, Semi,              // |, &, ;
    Newline,                      // \n
    LParen, RParen,              // ( )
    LBrace, RBrace,              // { }
    Less, Great,                 // <, >
    DLess, DGreat,               // <<, >>
    IO(u8),                      // 2>, 1>, etc.
    Bang,                        // !
    HereString,                  // <<< (optional)
    Eof,                         // end of input
}


pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens : Vec<Token> = vec![];
    let mut chars : Vec<char> = input.chars().collect();
    let mut inSingleQuote = false;
    let mut inDoubleQuote = false;
    let mut escaping = false;
    let mut token_buf = String::new();

    
    let mut i: usize = 0;

    while i < chars.len(){
        let c = chars[i];

        if escaping {
            token_buf.push(c);
            escaping = false;
            i +=1;
            continue;
        }

        if c == '\\' && !inSingleQuote{
            escaping = true;
            i+=1;
            continue;
        }
    }
        

    Ok(vec![])
}
