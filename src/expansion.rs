use crate::env::ShellEnv;

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticToken {
    Number(i64),
    Variable(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    Increment,
    Decrement,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,
    LParen,
    RParen,
    QuestionMark,
    Colon,
    Substitution(Vec<ArithmeticToken>),
}



pub fn expand(input: &str, env: &ShellEnv) -> String {
    // handle $VAR, ${VAR:-default}, $((1+2)), etc.
    return  String::from("");
}
