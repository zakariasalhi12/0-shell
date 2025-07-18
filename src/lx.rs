use crate::error::ShellError;
use std::iter::Peekable;
use std::str::Chars;



pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '\n' => {
                chars.next();
                tokens.push(Token::Newline);
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::AndIf);
                } else {
                    tokens.push(Token::Background);
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::OrIf);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            _ => {
                let word_parts = parse_word_parts(&mut chars)?;
                let word_str = word_parts_to_string(&word_parts);

                if let Some((name, value)) = parse_assignment(&word_str) {
                    tokens.push(Token::Assignment(name, value));
                } else {
                    tokens.push(Token::Word(word_parts));
                }
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

fn parse_word_parts<I>(chars: &mut Peekable<I>) -> Result<Vec<WordPart>, ShellError>
where
    I: Iterator<Item = char>,
{
    let mut parts = Vec::new();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '&' | '|' | ';' => break,
            '$' => {
                chars.next();
                match chars.peek() {
                    Some('(') => {
                        chars.next();
                        if chars.peek() == Some(&'(') {
                            chars.next();
                            let inner = extract_until_matching(chars, "))")?;
                            let arith_tokens = tokenize_arithmetic(&inner)?;
                            parts.push(WordPart::ArithmeticSubstitution(arith_tokens));
                        } else {
                            let inner = extract_until_matching(chars, ")")?;
                            let cmd_tokens = tokenize(&inner)?;
                            parts.push(WordPart::CommandSubstitution(cmd_tokens));
                        }
                    }
                    Some('{') => {
                        chars.next();
                        let var = extract_until_matching(chars, "}")?;
                        parts.push(WordPart::Variable(var));
                    }
                    Some(c) if is_var_char(*c) => {
                        let var = parse_variable(chars);
                        parts.push(WordPart::Variable(var));
                    }
                    _ => return Err(ShellError::InvalidVariableSyntax),
                }
            }
            '"' | '\'' => {
                let quote = chars.next().unwrap();
                let quoted = extract_until_matching(chars, &quote.to_string())?;
                parts.push(WordPart::Literal(quoted));
            }
            _ => {
                let literal = parse_literal(chars);
                parts.push(WordPart::Literal(literal));
            }
        }
    }

    Ok(parts)
}

fn parse_variable<I>(chars: &mut Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut var = String::new();
    while let Some(&c) = chars.peek() {
        if is_var_char(c) {
            var.push(c);
            chars.next();
        } else {
            break;
        }
    }
    var
}

fn is_var_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn parse_literal<I>(chars: &mut Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut lit = String::new();
    while let Some(&c) = chars.peek() {
        if c == '$' || c == '"' || c == '\'' || c.is_whitespace() || ['&', '|', ';', '\n'].contains(&c)
        {
            break;
        } else {
            lit.push(c);
            chars.next();
        }
    }
    lit
}

fn extract_until_matching<I>(chars: &mut Peekable<I>, end: &str) -> Result<String, ShellError>
where
    I: Iterator<Item = char>,
{
    let mut result = String::new();
    let mut depth = 1;
    let end_chars: Vec<char> = end.chars().collect();

    while let Some(c) = chars.next() {
        result.push(c);

        // Check for nested structures if end is multi-char
        if end.len() == 1 {
            if c == end_chars[0] {
                depth -= 1;
                if depth == 0 {
                    result.pop();
                    break;
                }
            } else if c == '(' {
                depth += 1;
            }
        } else if end == "))" {
            // Special handling for $(( ))
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                if let Some(&next_c) = chars.peek() {
                    if next_c == ')' {
                        chars.next();
                        result.pop();
                        break;
                    }
                }
                depth -= 1;
            }
        } else if c == end_chars[0] && end_chars.len() > 1 {
            // Could add more complex logic for other multi-char ends if needed
        }
    }

    if depth != 0 {
        Err(ShellError::Syntax("Unclosed delimiter".into()))
    } else {
        Ok(result)
    }
}

fn word_parts_to_string(parts: &[WordPart]) -> String {
    parts
        .iter()
        .map(|p| match p {
            WordPart::Literal(s) => s.clone(),
            WordPart::Variable(s) => format!("${}", s),
            WordPart::CommandSubstitution(_) => "$(...)".to_string(),
            WordPart::ArithmeticSubstitution(_) => "$((...))".to_string(),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn parse_assignment(word: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = word.splitn(2, '=').collect();
    if parts.len() == 2 && is_valid_varname(parts[0]) {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn is_valid_varname(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => (),
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn tokenize_arithmetic(input: &str) -> Result<Vec<ArithmeticToken>, ShellError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Number literal
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let num = num_str.parse::<i64>().map_err(|_| {
                ShellError::Syntax(format!("Invalid number: {}", num_str))
            })?;
            tokens.push(ArithmeticToken::Number(num));
            continue;
        }

        // Variable
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let var_str: String = chars[start..i].iter().collect();
            tokens.push(ArithmeticToken::Variable(var_str));
            continue;
        }

        // Nested arithmetic substitution $(( ... )) or (( ... ))
        if chars[i..].starts_with(&['$', '(', '(']) || chars[i..].starts_with(&['(', '(']) {
            let is_dollar = chars[i..].starts_with(&['$', '(', '(']);
            i += if is_dollar { 3 } else { 2 };

            let inner = extract_nested_expression(&chars, &mut i)?;
            let inner_tokens = tokenize_arithmetic(&inner)?;
            tokens.push(ArithmeticToken::Substitution(inner_tokens));
            continue;
        }

        // Multi-char operators
        let rest: String = chars[i..].iter().collect();
        let multi_ops = [
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
        ];

        if let Some((op_str, token)) = multi_ops.iter().find(|(op_str, _)| rest.starts_with(op_str)) {
            tokens.push(token.clone());
            i += op_str.len();
            continue;
        }

        // Single-char operators
        let single_token = match c {
            '+' => ArithmeticToken::Plus,
            '-' => ArithmeticToken::Minus,
            '*' => ArithmeticToken::Multiply,
            '/' => ArithmeticToken::Divide,
            '%' => ArithmeticToken::Modulo,
            '=' => ArithmeticToken::Assign,
            '<' => ArithmeticToken::Less,
            '>' => ArithmeticToken::Greater,
            '!' => ArithmeticToken::LogicalNot,
            '&' => ArithmeticToken::BitAnd,
            '|' => ArithmeticToken::BitOr,
            '^' => ArithmeticToken::BitXor,
            '~' => ArithmeticToken::BitNot,
            '?' => ArithmeticToken::QuestionMark,
            ':' => ArithmeticToken::Colon,
            '(' => ArithmeticToken::LParen,
            ')' => ArithmeticToken::RParen,
            _ => {
                return Err(ShellError::Syntax(format!("Unexpected character '{}'", c)));
            }
        };

        tokens.push(single_token);
        i += 1;
    }

    Ok(tokens)
}

fn extract_nested_expression(chars: &[char], i: &mut usize) -> Result<String, ShellError> {
    let mut depth = 1;
    let mut expr = String::new();

    while *i < chars.len() {
        let c = chars[*i];
        *i += 1;

        if c == '(' && *i < chars.len() && chars[*i] == '(' {
            depth += 1;
            expr.push(c);
            expr.push(chars[*i]);
            *i += 1;
            continue;
        }

        if c == ')' && *i < chars.len() && chars[*i] == ')' {
            depth -= 1;
            if depth == 0 {
                *i += 1;
                break;
            }
            expr.push(c);
            expr.push(chars[*i]);
            *i += 1;
            continue;
        }

        expr.push(c);
    }

    if depth != 0 {
        return Err(ShellError::Syntax("Unclosed arithmetic expression".to_string()));
    }

    Ok(expr)
}
