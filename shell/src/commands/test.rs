use crate::ShellCommand;
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use std::io::{self, ErrorKind};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct Test {
    pub args: Vec<String>,
    pub is_bracket: bool,
}

impl Test {
    pub fn new(args: Vec<String>, is_bracket: bool) -> Self {
        Self { args, is_bracket }
    }

    pub fn parse_args(&self) -> Result<TestArgs, ShellError> {
        let args = if self.is_bracket {
            if self.args.is_empty() {
                return Err(ShellError::Exec("[ : needs closing ]".to_string()));
            }
            if self.args.last() != Some(&"]".to_string()) {
                return Err(ShellError::Exec("[ : needs closing ]".to_string()));
            }
            &self.args[..self.args.len() - 1]
        } else {
            &self.args[..]
        };

        Ok(TestArgs { args })
    }
}

struct TestArgs<'a> {
    args: &'a [String],
}

impl<'a> TestArgs<'a> {
    fn unary_test(&self, op: &str, arg: &str) -> io::Result<i32> {
        match op {
            "-n" => Ok(if !arg.is_empty() { 0 } else { 1 }),
            "-z" => Ok(if arg.is_empty() { 0 } else { 1 }),
            "-d" => Ok(if Path::new(arg).is_dir() { 0 } else { 1 }),
            "-e" => Ok(if Path::new(arg).exists() { 0 } else { 1 }),
            "-f" => Ok(if Path::new(arg).is_file() { 0 } else { 1 }),
            "-r" => Ok(if Path::new(arg).exists() { 0 } else { 1 }),
            "-w" => Ok(if Path::new(arg).exists() { 0 } else { 1 }),
            "-x" => Ok(if Path::new(arg).exists() { 0 } else { 1 }),
            _ => Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("test: unknown unary operator '{}'", op),
            )),
        }
    }

    fn binary_test(&self, left: &str, op: &str, right: &str) -> io::Result<i32> {
        match op {
            "=" => Ok(if left == right { 0 } else { 1 }),
            "!=" => Ok(if left != right { 0 } else { 1 }),
            "-eq" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left == right { 0 } else { 1 })
            }
            "-ne" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left != right { 0 } else { 1 })
            }
            "-lt" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left < right { 0 } else { 1 })
            }
            "-le" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left <= right { 0 } else { 1 })
            }
            "-gt" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left > right { 0 } else { 1 })
            }
            "-ge" => {
                let left = left.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                let right = right.parse::<i64>().map_err(|_| {
                    io::Error::new(ErrorKind::InvalidInput, "test: invalid integer")
                })?;
                Ok(if left >= right { 0 } else { 1 })
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("test: unknown binary operator '{}'", op),
            )),
        }
    }

    fn evaluate(&self) -> io::Result<i32> {
        match self.args.len() {
            0 => Ok(1), // false
            1 => Ok(if self.args[0].is_empty() { 1 } else { 0 }),
            2 => self.unary_test(&self.args[0], &self.args[1]),
            3 => self.binary_test(&self.args[0], &self.args[1], &self.args[2]),
            _ => Err(io::Error::new(
                ErrorKind::InvalidInput,
                "test: too many arguments",
            )),
        }
    }
}

impl ShellCommand for Test {
    fn execute(&self, env: &mut ShellEnv) -> Result<i32, ShellError> {
        let test_args = self.parse_args().map_err(|e| {
            io::Error::new(ErrorKind::InvalidInput, e.to_string())
        })?;
        test_args.evaluate()
    }
}