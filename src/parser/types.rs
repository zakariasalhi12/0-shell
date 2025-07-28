// src/ast.rs
use std::fmt;
use crate::lexer::types::{Word, WordPart};

// Arithmetic expression AST
#[derive(Debug, Clone)]
pub enum ArithmeticExpr {
    Literal(i64),
    Variable(String),
    UnaryOp {
        op: UnaryOperator,
        expr: Box<ArithmeticExpr>,
    },
    BinaryOp {
        op: BinaryOperator,
        lhs: Box<ArithmeticExpr>,
        rhs: Box<ArithmeticExpr>,
    },
    Assignment {
        var: String,
        value: Box<ArithmeticExpr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Negate,     // -x
    Not,        // !x
    BitNot,     // ~x
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Eq,         // ==
    Neq,        // !=
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=

    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    ShiftLeft,  // <<
    ShiftRight, // >>

    LogicalAnd, // &&
    LogicalOr,  // ||
}

// I/O Redirection support
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectOp {
    /// `>`: redirect stdout to a file (overwrite)
    Write,
    /// `>>`: redirect stdout to a file (append)
    Append,
    /// `<`: redirect stdin from a file
    Read,
    /// `<<`: here-document
    HereDoc,
    /// `<>`: open file for read and write
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect{
    pub fd : Option<u64>,
    pub kind: RedirectOp,
    pub target : Word,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Command {
        cmd: Word,
        args : Vec<Word>,
        assignments: Vec<(String, Vec<WordPart>)>,
        redirects: Vec<Redirect>,
    },

    Pipeline(Vec<AstNode>),
    Sequence(Vec<AstNode>),
    And(Box<AstNode>, Box<AstNode>),
    Or(Box<AstNode>, Box<AstNode>),
    Not(Box<AstNode>),
    Background(Box<AstNode>),

    Subshell(Box<AstNode>),
    Group{
        commands : Vec<AstNode>,
        redirects : Vec<Redirect>
    },

    If {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    Until {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    For {
        var: String,
        values: Vec<String>,
        body: Box<AstNode>,
    },
    Case {
        word: String,
        arms: Vec<(Vec<String>, AstNode)>,
    },
    FunctionDef {
        name: Word,
        body: Box<AstNode>,
    },

    ArithmeticCommand(ArithmeticExpr),
}


impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

impl AstNode {
    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let spaces = "  ".repeat(indent);
        
        match self {
            AstNode::Command { cmd, args, assignments, redirects } => {
                writeln!(f, "{}Command", spaces)?;
                writeln!(f, "{}  cmd: {:?}", spaces, cmd)?;
                
                if !assignments.is_empty() {
                    writeln!(f, "{}  assignments:", spaces)?;
                    for (key, parts) in assignments {
                        writeln!(f, "{}    {} = {:?}", spaces, key, parts)?;
                    }
                }
                
                if !args.is_empty() {
                    writeln!(f, "{}  args:", spaces)?;
                    for arg in args {
                        writeln!(f, "{}    {:?}", spaces, arg)?;
                    }
                }
                
                if !redirects.is_empty() {
                    writeln!(f, "{}  redirects:", spaces)?;
                    for redirect in redirects {
                        writeln!(f, "{}    {}", spaces, redirect)?;
                    }
                }
            }
            
            AstNode::Pipeline(commands) => {
                writeln!(f, "{}Pipeline", spaces)?;
                for (i, cmd) in commands.iter().enumerate() {
                    writeln!(f, "{}  [{}]", spaces, i)?;
                    cmd.fmt_with_indent(f, indent + 2)?;
                }
            }
            
            AstNode::And(left, right) => {
                writeln!(f, "{}And (&&)", spaces)?;
                writeln!(f, "{}  left:", spaces)?;
                left.fmt_with_indent(f, indent + 2)?;
                writeln!(f, "{}  right:", spaces)?;
                right.fmt_with_indent(f, indent + 2)?;
            }
            
            AstNode::Or(left, right) => {
                writeln!(f, "{}Or (||)", spaces)?;
                writeln!(f, "{}  left:", spaces)?;
                left.fmt_with_indent(f, indent + 2)?;
                writeln!(f, "{}  right:", spaces)?;
                right.fmt_with_indent(f, indent + 2)?;
            }
            
            _ => {
                writeln!(f, "{}{:?}", spaces, self)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Redirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Redirect(fd: {:?}, kind: {:?}, target: {:?})", 
               self.fd, self.kind, self.target)
    }
}