use crate::ShellCommand;
use crate::envirement::ShellEnv;
use crate::error::ShellError;
use std::path::Path;
use std::fs;
use std::os::unix::fs::MetadataExt; 
use libc::{geteuid, getegid};

#[derive(Debug, PartialEq, Eq)]
pub struct Test {
    pub args: Vec<String>,
    pub is_bracket: bool,
}

impl Test {
    pub fn new(args: Vec<String>, is_bracket: bool) -> Self {
        Self { args, is_bracket }
    }

    fn parse_args(&self) -> Result<TestArgs, ShellError> {
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
    fn unary_test(&self, op: &str, arg: &str) -> Result<i32, ShellError> {
        match op {
            "-n" => Ok(if !arg.is_empty() { 0 } else { 1 }),
            "-z" => Ok(if arg.is_empty() { 0 } else { 1 }),
            "-d" => Ok(if Path::new(arg).is_dir() { 0 } else { 1 }),
            "-e" => Ok(if Path::new(arg).exists() { 0 } else { 1 }),
            "-f" => Ok(if Path::new(arg).is_file() { 0 } else { 1 }),
            "-r" => Ok(if Self::check_permission(arg, 0o4) { 0 } else { 1 }), // read
            "-w" => Ok(if Self::check_permission(arg, 0o2) { 0 } else { 1 }), // write
            "-x" => Ok(if Self::check_permission(arg, 0o1) { 0 } else { 1 }), // execute

            _ => Err(ShellError::Exec(format!("test: unknown unary operator '{}'", op))),
        }
    }

    fn binary_test(&self, left: &str, op: &str, right: &str) -> Result<i32, ShellError> {
        match op {
            "=" => Ok(if left == right { 0 } else { 1 }),
            "!=" => Ok(if left != right { 0 } else { 1 }),
            "-eq" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left == right { 0 } else { 1 })
            }
            "-ne" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left != right { 0 } else { 1 })
            }
            "-lt" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left < right { 0 } else { 1 })
            }
            "-le" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left <= right { 0 } else { 1 })
            }
            "-gt" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left > right { 0 } else { 1 })
            }
            "-ge" => {
                let left = left.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                let right = right.parse::<i64>().map_err(|_| ShellError::Exec("test: invalid integer".to_string()))?;
                Ok(if left >= right { 0 } else { 1 })
            }
            _ => Err(ShellError::Exec(format!("test: unknown binary operator '{}'", op))),
        }
    }

    fn evaluate(&self) -> Result<i32, ShellError> {
        match self.args.len() {
            0 => Ok(1),
            1 => Ok(if self.args[0].is_empty() { 1 } else { 0 }),
            2 => self.unary_test(&self.args[0], &self.args[1]),
            3 => self.binary_test(&self.args[0], &self.args[1], &self.args[2]),
            _ => Err(ShellError::Exec("test: too many arguments".to_string())),
        }
    }

    fn check_permission(path: &str, perm_bit: u32) -> bool {
        let path = Path::new(path);
        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.mode(); 
            let file_uid = metadata.uid();
            let file_gid = metadata.gid();
            let euid = unsafe { geteuid() }; 
            let egid = unsafe { getegid() }; 

            let bits = if euid == 0 {
                0o777
            } else if euid == file_uid {
                (mode >> 6) & 0o7
            } else if egid == file_gid {
                (mode >> 3) & 0o7
            } else {
                mode & 0o7
            };

            (bits & perm_bit) != 0
        } else {
            false
        }
    }
}

impl ShellCommand for Test {
    fn execute(&self, _env: &mut ShellEnv) -> Result<i32, ShellError> {
        let test_args = self.parse_args()?;
        test_args.evaluate()
    }
}