use crate::error::ShellError;
use crate::expansion::ArithmeticToken;

#[derive(Debug, Clone)]
pub enum Token {
    Word(String), // any word or argument
    Variable(String),
    CommandSubstitution(Vec<Token>),              // $(...)
    ArithmeticSubstitution(Vec<ArithmeticToken>), // $((...))
    Assignment(String, String),                   // FOO=bar
    AndIf,
    OrIf, // &&, ||
    Pipe,
    Amp,
    Semi,    // |, &, ;
    Newline, // \n
    LParen,
    RParen, // ( )
    LBrace,
    RBrace, // { }
    Less,
    Great, // <, >
    DLess,
    DGreat,     // <<, >>
    IO(u8),     // 2>, 1>, etc.
    Bang,       // !
    HereString, // <<< (optional)
    Eof,        // end of input
    RedirectDuplicateOut,
    RedirectDuplicateIn,
}

// trait Substitutable {
//     fn parse(input: &str) -> Result<Vec<Self>, ShellError>
//     where
//         Self: Sized;
// }

// impl Substitutable for Token {
//     fn parse(input: &str) -> Result<Vec<Self>, ShellError> {
//         return tokenize(input);
//     }
// }

// impl Substitutable for ArithmeticToken {
//     fn parse(input: &str) -> Result<Vec<Self>, ShellError> {
//         return tokenize_arithmetic(input);
//     }
// }

pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens: Vec<Token> = vec![];
    let chars: Vec<char> = input.chars().collect();
    let mut inSingleQuote = false;
    let mut inDoubleQuote = false;
    let mut escaping = false;
    let mut token_buf = String::new();
    let mut command_sub = false;
    let mut arithmetic_sub = false;

    let mut i: usize = 0;

    while i < chars.len() {
        let c = chars[i];

        if escaping {
            token_buf.push(c);
            escaping = false;
            i += 1;
            continue;
        }

        if c == '\\' && !inSingleQuote {
            escaping = true;
            i += 1;
            continue;
        }

        if c == '\'' && !inDoubleQuote {
            inSingleQuote = !inSingleQuote;
            i += 1;
            continue;
        }

        if c == '"' && !inSingleQuote {
            inDoubleQuote = !inDoubleQuote;
            i += 1;
            continue;
        }

        if inSingleQuote || inDoubleQuote {
            token_buf.push(c);
            i += 1;
            continue;
        }

        if c.is_whitespace() {
            if !token_buf.is_empty() {
                tokens.push(Token::Word(token_buf.clone()));
                token_buf.clear();
            }

            if c == '\n' {
                tokens.push(Token::Newline);
            }

            i += 1;
            continue;
        }

        if let Some(three) = chars.get(i..=i + 2) {
            let three: String = three.iter().collect();

            match three.as_str() {
                "<<<" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::HereString);
                    i += 3;
                    continue;
                }
                "$((" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    i += 3;

                    let mut depth = 1;
                    let mut expression = String::new();

                    while i + 1 < chars.len() {
                        if chars[i] == '(' && chars[i + 1] == '(' {
                            depth += 1;
                            expression.push('(');
                            expression.push('(');
                            i += 2;
                            continue;
                        }

                        if chars[i] == ')' && chars[i + 1] == ')' {
                            depth -= 1;
                            if depth == 0 {
                                i += 2;
                                break;
                            }
                            expression.push(')');
                            expression.push(')');
                            i += 2;
                            continue;
                        }

                        expression.push(chars[i]);
                        i += 1;
                    }

                    if depth != 0 {
                        return Err(ShellError::Syntax(
                            "Unclosed arithmetic substitution".to_string(),
                        ));
                    }

                    match tokenize_arithmetic(&expression) {
                        Ok(subtokens) => tokens.push(Token::ArithmeticSubstitution(subtokens)),
                        Err(e) => return Err(e),
                    }

                    continue;
                }
                _ => {}
            }
        }

        if let Some(two) = chars.get(i..=i + 1) {
            let two: String = two.iter().collect();

            match two.as_str() {
                "&&" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::AndIf);
                    i += 2;
                    continue;
                }
                "||" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::OrIf);
                    i += 2;
                    continue;
                }
                ">>" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::DGreat);
                    i += 2;
                    continue;
                }
                "<<" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::DLess);
                    i += 2;
                    continue;
                }
                ">&" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::RedirectDuplicateOut);
                    i += 2;
                    continue;
                }
                "<&" => {
                    flush_buf(&mut token_buf, &mut tokens);
                    tokens.push(Token::RedirectDuplicateIn);
                    i += 2;
                    continue;
                }
                ";;" => {}
                "$(" => {
                    if !inSingleQuote {
                        flush_buf(&mut token_buf, &mut tokens);
                        i += 2;

                        let mut depth = 1;
                        let mut expression = String::new();

                        while i + 1 < chars.len() {
                            if chars[i] == '(' && chars[i + 1] == '(' {
                                depth += 1;
                                expression.push('(');
                                expression.push('(');
                                i += 2;
                                continue;
                            }

                            if chars[i] == ')' && chars[i + 1] == ')' {
                                depth -= 1;
                                if depth == 0 {
                                    i += 2;
                                    break;
                                }
                                expression.push(')');
                                expression.push(')');
                                i += 2;
                                continue;
                            }

                            expression.push(chars[i]);
                            i += 1;
                        }

                        if depth != 0 {
                            return Err(ShellError::Syntax(
                                "Unclosed arithmetic substitution".to_string(),
                            ));
                        }

                        match tokenize(&expression) {
                            Ok(subtokens) => tokens.push(Token::CommandSubstitution(subtokens)),
                            Err(e) => return Err(e),
                        }

                        continue;
                    }
                }

                _ => {}
            }
        }

        match c {
            '|' => {}
            '&' => {}
            ';' => {}
            '<' => {}
            '>' => {}
            '!' => {}
            '(' => {}
            ')' => {
                if command_sub {
                    command_sub = !command_sub;
                    if let Ok(res) = tokenize(&token_buf) {
                        tokens.push(Token::CommandSubstitution(res));
                        token_buf.clear();
                    } else {
                        return Err(ShellError::Syntax("".to_string()));
                    }
                    i += 1;
                    continue;
                } else if arithmetic_sub {
                    arithmetic_sub = !arithmetic_sub;
                    if let Ok(res) = tokenize_arithmetic(&token_buf) {
                        tokens.push(Token::ArithmeticSubstitution(res));
                        token_buf.clear();
                    } else {
                        return Err(ShellError::Syntax("".to_string()));
                    }
                    i += 1;
                }
            }
            '{' => {}
            '}' => {}
            '$' => {}
            _ => {}
        }
    }

    Ok(vec![])
}

fn flush_buf(buf: &mut String, tokens: &mut Vec<Token>) {
    if !buf.is_empty() {
        tokens.push(Token::Word(buf.clone()));
        buf.clear();
    }
}

pub fn tokenize_arithmetic(input: &str) -> Result<Vec<ArithmeticToken>, ShellError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = vec![];
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() {
            let mut num = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            if let Ok(n) = num.parse::<i64>() {
                tokens.push(ArithmeticToken::Number(n));
            }
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let mut var = String::new();
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                var.push(chars[i]);
                i += 1;
            }
            tokens.push(ArithmeticToken::Variable(var));
            continue;
        }

        if chars[i..].starts_with(&['$', '(', '(']) || chars[i..].starts_with(&['(', '(']) {
            let is_dollar = chars[i..].starts_with(&['$', '(', '(']);
            i += if is_dollar { 3 } else { 2 };

            let mut depth = 1;
            let mut expression = String::new();

            while i + 1 < chars.len() {
                if chars[i] == '(' && chars[i + 1] == '(' {
                    depth += 1;
                    expression.push('(');
                    expression.push('(');
                    i += 2;
                    continue;
                }

                if chars[i] == ')' && chars[i + 1] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        i += 2;
                        break;
                    }
                    expression.push(')');
                    expression.push(')');
                    i += 2;
                    continue;
                }

                expression.push(chars[i]);
                i += 1;
            }

            if depth != 0 {
                return Err(ShellError::Syntax(
                    "Unclosed arithmetic expression".to_string(),
                ));
            }

            match tokenize_arithmetic(&expression) {
                Ok(tks) => tokens.push(ArithmeticToken::Substitution(tks)),
                Err(e) => return Err(e),
            }
            continue;
        }

        let rest: String = chars[i..].iter().collect();
        let matched = [
            ("++", ArithmeticToken::Increment),
            ("--", ArithmeticToken::Decrement),
            ("+=", ArithmeticToken::AddAssign),
            ("-=", ArithmeticToken::SubAssign),
            ("*=", ArithmeticToken::MulAssign),
            ("/=", ArithmeticToken::DivAssign),
            ("%=", ArithmeticToken::ModAssign),
            ("==", ArithmeticToken::Equal),
            ("!=", ArithmeticToken::NotEqual),
            ("<=", ArithmeticToken::LessEqual),
            (">=", ArithmeticToken::GreaterEqual),
            ("&&", ArithmeticToken::LogicalAnd),
            ("||", ArithmeticToken::LogicalOr),
            ("<<", ArithmeticToken::ShiftLeft),
            (">>", ArithmeticToken::ShiftRight),
        ]
        .iter()
        .find(|(pat, _)| rest.starts_with(*pat));

        if let Some((pat, token)) = matched {
            tokens.push(token.clone());
            i += pat.len();
            continue;
        }

        let single = match c {
            '+' => Some(ArithmeticToken::Plus),
            '-' => Some(ArithmeticToken::Minus),
            '*' => Some(ArithmeticToken::Multiply),
            '/' => Some(ArithmeticToken::Divide),
            '%' => Some(ArithmeticToken::Modulo),
            '=' => Some(ArithmeticToken::Assign),
            '<' => Some(ArithmeticToken::Less),
            '>' => Some(ArithmeticToken::Greater),
            '!' => Some(ArithmeticToken::LogicalNot),
            '&' => Some(ArithmeticToken::BitAnd),
            '|' => Some(ArithmeticToken::BitOr),
            '^' => Some(ArithmeticToken::BitXor),
            '~' => Some(ArithmeticToken::BitNot),
            '?' => Some(ArithmeticToken::QuestionMark),
            ':' => Some(ArithmeticToken::Colon),
            '(' => Some(ArithmeticToken::LParen),
            ')' => Some(ArithmeticToken::RParen),
            _ => None,
        };

        if let Some(token) = single {
            tokens.push(token);
            i += 1;
        } else {
            return Err(ShellError::Syntax(
                "Unexpected arithmetic token".to_string(),
            ));
        }
    }

    Ok(tokens)
}

// fn handle_substitution<T: Substitutable>(rest: &str) -> Result<Vec<T>, ShellError> {
//     return Ok(vec![]);
// }
