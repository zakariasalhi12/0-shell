use crate::{envirement::ShellEnv, lexer::types::Word};
use std::fmt;

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
    Negate, // -x
    Not,    // !x
    BitNot, // ~x
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Eq,  // ==
    Neq, // !=
    Lt,  // <
    Gt,  // >
    Le,  // <=
    Ge,  // >=

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
pub struct Redirect {
    pub fd: Option<u64>,
    pub kind: RedirectOp,
    pub target: Word,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Command {
        cmd: Word,
        args: Vec<Word>,
        assignments: Vec<(String, Word)>,
        redirects: Vec<Redirect>,
    },

    Pipeline(Vec<AstNode>),
    Sequence(Vec<AstNode>),
    And(Box<AstNode>, Box<AstNode>),
    Or(Box<AstNode>, Box<AstNode>),
    Not(Box<AstNode>),
    Background(Box<AstNode>),

    Subshell(Box<AstNode>),
    Group {
        commands: Vec<AstNode>,
        redirects: Vec<Redirect>,
    },

    If {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        elif: Vec<(Box<AstNode>, Box<AstNode>)>,
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
        values: Vec<Word>,
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
    Break(Option<Word>),
    Continue(Option<Word>),
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
            AstNode::Command {
                cmd,
                args,
                assignments,
                redirects,
            } => {
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
        write!(
            f,
            "Redirect(fd: {:?}, kind: {:?}, target: {:?})",
            self.fd, self.kind, self.target
        )
    }
}

impl Redirect {
    pub fn to_text(&self, env: &ShellEnv) -> String {
        let fd_str = self.fd.map(|fd| fd.to_string()).unwrap_or_default();
        let op_str = match self.kind {
            RedirectOp::Write => ">",
            RedirectOp::Append => ">>",
            RedirectOp::Read => "<",
            RedirectOp::HereDoc => "<<",
            RedirectOp::ReadWrite => "<>",
        };
        format!("{}{}{}", fd_str, op_str, self.target.expand(env))
    }
}

impl AstNode {
    pub fn to_text(&self, env: &ShellEnv) -> String {
        match self {
            AstNode::Command { cmd, args, assignments, redirects } => {
                let mut parts = Vec::new();
                for (k, v) in assignments {
                    parts.push(format!("{}={}", k, v.expand(env)));
                }
                parts.push(cmd.expand(env));
                for arg in args {
                    parts.push(arg.expand(env));
                }
                for r in redirects {
                    parts.push(r.to_text(env));
                }
                parts.join(" ")
            }

            AstNode::Pipeline(nodes) => 
                nodes.iter().map(|n| n.to_text(env)).collect::<Vec<_>>().join(" | "),

            AstNode::Sequence(nodes) => 
                nodes.iter().map(|n| n.to_text(env)).collect::<Vec<_>>().join("; "),

            AstNode::And(lhs, rhs) => 
                format!("{} && {}", lhs.to_text(env), rhs.to_text(env)),

            AstNode::Or(lhs, rhs) => 
                format!("{} || {}", lhs.to_text(env), rhs.to_text(env)),

            AstNode::Not(node) => 
                format!("! {}", node.to_text(env)),

            AstNode::Background(node) => 
                format!("{} &", node.to_text(env)),

            AstNode::Subshell(node) => 
                format!("(${})", node.to_text(env)),

            AstNode::Group { commands, redirects } => {
                let mut s = format!(
                    "{{ {}; }}",
                    commands.iter().map(|c| c.to_text(env)).collect::<Vec<_>>().join("; ")
                );
                for r in redirects {
                    s.push_str(&format!(" {}", r.to_text(env)));
                }
                s
            }

            AstNode::If { condition, then_branch, elif, else_branch } => {
                let mut s = format!("if {}; then {}", condition.to_text(env), then_branch.to_text(env));
                for (cond, body) in elif {
                    s.push_str(&format!(" elif {}; then {}", cond.to_text(env), body.to_text(env)));
                }
                if let Some(else_b) = else_branch {
                    s.push_str(&format!(" else {}", else_b.to_text(env)));
                }
                s.push_str(" fi");
                s
            }

            AstNode::While { condition, body } => 
                format!("while {}; do {}; done", condition.to_text(env), body.to_text(env)),

            AstNode::Until { condition, body } => 
                format!("until {}; do {}; done", condition.to_text(env), body.to_text(env)),

            AstNode::For { var, values, body } => {
                // let vals = values.iter().reduce(|w1, w2| {
                //     return w1.expand(env);
                // });
                // format!("for {} in {}; do {}; done", "".to_string(),"".to_string(), "".to_string() body.to_text(env))
                return String::new();
            }

            AstNode::Case { word, arms } => {
                let mut s = format!("case {} in", word);
                for (pats, body) in arms {
                    let pat_str = pats.join(" | ");
                    s.push_str(&format!(" {} ) {} ;;", pat_str, body.to_text(env)));
                }
                s.push_str(" esac");
                s
            }

            AstNode::FunctionDef { name, body } => 
                format!("{}() {{ {}; }}", name.expand(env), body.to_text(env)),

          _ =>{
                format!("{:?}", self)
          }}
    }
}
