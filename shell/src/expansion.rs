// use crate::{envirement::ShellEnv, lexer::{self, types::{QuoteType, Word, WordPart}}, Parser};


// impl Word{
//     pub fn expand(self, env: &ShellEnv) -> String {
//     for part in self.parts{
//         match (part, self.quote){
//             (WordPart::CommandSubstitution(expression), QuoteType::Double | QuoteType::None) =>{
//                 // Parser::new(lexer::tokenize::Tokenizer::new(&expression))
                

//             },
//             (WordPart::VariableSubstitution(var), QuoteType::Double | QuoteType::None) =>{

//             },
//             (WordPart::ArithmeticSubstitution(expression), QuoteType::Double | QuoteType::None)=>{

//             },
//             _ =>{

//             }
//         }
//     }
//     return  String::from("");
// }

// }
