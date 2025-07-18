// use std::fmt::Error;

// use crate::ast::*;
// use crate::lexer::Token;
// use crate::error::ShellError;

// pub struct Parser{
//     tokens : Vec<Token>,
//     pos : usize,
// }

// impl Parser{
//     pub fn new(tokens: Vec<Token>)-> Self{
//         return Self{tokens, pos :0};
//     }

//     pub fn peek(&self) -> Option<&Token>{
//         self.tokens.get(self.pos)
//     }

//     pub fn next(&mut self) -> Option<&Token>{
//         let tok = self.tokens.get(self.pos);
//         self.pos+=1;
//         tok
//     }

//     pub fn check(&self, expected : &Token) -> bool{
//         if let Some(tok) = self.peek(){
//             return tok == expected
//         }
//         return false;
//     }

//     pub fn eat(&self, expected : &Token) -> Result<Token, ShellError>{
//         if self.check(expected){
//             return Ok(expected.to_owned());
//         }
//         Err(ShellError::Parse(format!("unexpected token: {:#?}, after: {:#?}",self.peek() , expected)))
//     }
//     pub fn skip_whitespaces(&mut self){
        
//     }


// }

// pub fn parse(tokens: &[Token]) -> Result<AstNode, ShellError> {
//     // TODO: implement recursive descent parser
//     Err(ShellError::Parse("unimplemented parser".into()))
// }


// // pub fn parse_simple_command(str : &str) -> 