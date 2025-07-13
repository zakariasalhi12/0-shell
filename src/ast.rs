// src/ast.rs

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
#[derive(Debug, Clone)]
pub enum RedirectKind {
    Input,              // <
    Output,             // >
    Append,             // >>
    HereDoc,            // <<
    HereString,         // <<<
    DupInput,           // <&n
    DupOutput,          // >&n
}

#[derive(Debug, Clone)]
pub struct Redirect {
    pub fd: Option<u8>,
    pub kind: RedirectKind,
    pub target: String,
}

// Main AST Node
#[derive(Debug, Clone)]
pub enum AstNode {
    SimpleCommand {
        assignments: Vec<(String, String)>,
        words: Vec<String>,
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
