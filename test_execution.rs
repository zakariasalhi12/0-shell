use shell::exec::execute;
use shell::envirement::ShellEnv;
use shell::parser::types::*;
use shell::lexer::types::{Word, WordPart, QuoteType};

fn main() {
    // Test basic command execution
    let mut env = ShellEnv::new();
    
    // Test echo command
    let echo_cmd = AstNode::Command {
        cmd: Word {
            parts: vec![WordPart::Literal("echo".to_string())],
            quote: QuoteType::None,
        },
        args: vec![
            Word {
                parts: vec![WordPart::Literal("Hello".to_string())],
                quote: QuoteType::None,
            },
            Word {
                parts: vec![WordPart::Literal("World".to_string())],
                quote: QuoteType::None,
            },
        ],
        assignments: vec![],
        redirects: vec![],
    };
    
    println!("Testing echo command...");
    let result = execute(&echo_cmd, &mut env);
    println!("Result: {:?}", result);
    
    // Test variable assignment
    let assign_cmd = AstNode::Command {
        cmd: Word {
            parts: vec![WordPart::Literal("".to_string())],
            quote: QuoteType::None,
        },
        args: vec![],
        assignments: vec![
            ("TEST_VAR".to_string(), vec![WordPart::Literal("test_value".to_string())]),
        ],
        redirects: vec![],
    };
    
    println!("Testing variable assignment...");
    let result = execute(&assign_cmd, &mut env);
    println!("Result: {:?}", result);
    println!("TEST_VAR = {:?}", env.get_var("TEST_VAR"));
    
    // Test sequence
    let seq_cmd = AstNode::Sequence(vec![
        echo_cmd.clone(),
        assign_cmd.clone(),
    ]);
    
    println!("Testing sequence...");
    let result = execute(&seq_cmd, &mut env);
    println!("Result: {:?}", result);
} 