#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub parts: Vec<WordPart>,
    pub quote: QuoteType,
}

use crate::{envirement::ShellEnv};


impl Word{
    pub fn expand(&self, env: &ShellEnv) -> String {
        let mut result = String::new();
            for part in &self.parts{
                match (part, self.quote){
                    (WordPart::CommandSubstitution(expression), QuoteType::Double | QuoteType::None) =>{
                        // Parser::new(lexer::tokenize::Tokenizer::new(&expression))
                       if let Some(value) = env.get("0"){
                            
                        }
                        

                    },
                    (WordPart::VariableSubstitution(var), QuoteType::Double | QuoteType::None) =>{
                        if let Some(value) = env.get(&var){
                            result.push_str(&value);
                        }
                    },
                    (WordPart::ArithmeticSubstitution(expression), QuoteType::Double | QuoteType::None)=>{
                    },
                    _ =>{

                    }
                }
            }
    return  String::from("");
}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(Word),
    Pipe,
    RedirectIn,           
    RedirectOut,
    RedirectAppend,       
    RedirectInFd(u64),    
    RedirectOutFd(u64),  
    RedirectAppendFd(u64),
    RedirectHereDoc,
    Semicolon,
    Ampersand,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum QuoteType {
    Single,
    Double,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WordPart {
    Literal(String),
    VariableSubstitution(String),   // $USER
    ArithmeticSubstitution(String), // $((1 + 2))
    CommandSubstitution(String),    // $(whoami)
}

#[derive(Debug)]
pub enum State {
    Default,
    InWord,
    InDoubleQuote,
    InSingleQuote,
    MaybeRedirectOut2,
    MaybeRedirectIn2,
    MaybeRedirectOut2Fd(u64),
    MaybeRedirectIn2Fd(u64),


}