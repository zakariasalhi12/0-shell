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
    let chars : Vec<char> = input.chars().collect();
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

        if c == '\'' && !inDoubleQuote{
            inSingleQuote = !inSingleQuote;
            i+=1;
            continue;
        }

        if c == '"' && !inSingleQuote{
            inDoubleQuote = !inDoubleQuote;
            i+=1;
            continue;
        }

        if inSingleQuote || inDoubleQuote{
            token_buf.push(c);
            i+=1;
            continue;
        }


        if c.is_whitespace(){
            if !token_buf.is_empty(){
                tokens.push(Token::Word(token_buf.clone()));
                token_buf.clear();
            }

            if c == '\n'{
                tokens.push(Token::Newline);
            }

            i+=1;
            continue;
        }

        if let Some(three) = chars.get(i ..= i+2){
            let three : String = three.iter().collect();

            if three == "<<<"{
                flush_buf(&mut token_buf, &mut tokens);
                tokens.push(Token::HereString);
                i+=3;
                continue;
            }
        }

        if let Some(two) = chars.get(i ..= i+1){
            let two : String = two.iter().collect();

            match two.as_str(){
                "&&" =>{

                },
                "||" => {

                },
                ">>" => {

                },
                "<<" => {

                },
                ">&" => {

                },
                "<&" => {

                },
                ";;" => {

                },
                              
                _ =>{

                }
            }
        }

       match c {
    '|' => {},
    '&' => {},
    ';' => {},
    '<' => {},
    '>' => {},
    '!' => {},
    '(' => {},
    ')' => {},
    '{' => {},
    '}' => {},
    '$' => {},
    _ => {},
}

    }
        

    Ok(vec![])
}


fn flush_buf(buf: &mut String, tokens: &mut Vec<Token>) {
    if !buf.is_empty() {
        tokens.push(Token::Word(buf.clone()));
        buf.clear();
    }
}