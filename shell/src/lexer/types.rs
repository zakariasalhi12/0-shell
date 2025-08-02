#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub parts: Vec<WordPart>,
    pub quote: QuoteType,
}

use std::collections::HashMap;

use crate::{envirement::ShellEnv};


impl Word{
    pub fn expand(&self, env: &ShellEnv, scoped_env : &HashMap<String, String>) -> String {
        let mut result = String::new();
            for part in &self.parts{
                match (part, self.quote){
                    (WordPart::CommandSubstitution(expression), QuoteType::Double | QuoteType::None) =>{
                        // Parser::new(lexer::tokenize::Tokenizer::new(&expression))
                       if let Some(value) = env.get("0"){
                            
                        }
                        

                    },
                    (WordPart::CommandSubstitution(word), QuoteType::Single) =>{
                            result.push_str(&word);
                    },
                    (WordPart::VariableSubstitution(var), QuoteType::Double | QuoteType::None) =>{
                        if let Some(value) = scoped_env.get(var){
                            result.push_str(&value);
                        } else if let Some(value) = env.get(&var){
                            result.push_str(&value);
                        }
                    },
                    (WordPart::VariableSubstitution(word), QuoteType::Single) =>{
                            result.push_str(&word);
                    },
                    (WordPart::ArithmeticSubstitution(expression), QuoteType::Double | QuoteType::None)=>{
                        
                    },
                    (WordPart::ArithmeticSubstitution(word), QuoteType::Single) =>{
                            result.push_str(&word);
                    },
                    (WordPart::Literal(word), _) =>{
                       result.push_str(&word);
                    }
                }
            }
    return  result;
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