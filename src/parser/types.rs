// src/ast.rs

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
    /// `>&`: redirect stdout to another FD
    DupWrite, // e.g., `2>&1`
    /// `<&`: redirect stdin from another FD
    DupRead, // e.g., `0<&1`
    /// `>&-` or `<&-`: close FD
    CloseFd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    File(Word),      // Regular file target
    Fd(u64),         // File descriptor target (e.g., &1, &2)
    Close,           // Close file descriptor target (&-)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect{
    pub fd : Option<u64>,
    pub kind: RedirectOp,
    pub target : RedirectTarget,
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
    Group(Box<AstNode>),

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
        name: String,
        body: Box<AstNode>,
    },

    ArithmeticCommand(ArithmeticExpr),
}
