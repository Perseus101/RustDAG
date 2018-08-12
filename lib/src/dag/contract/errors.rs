use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ArgLenMismatchError;

impl fmt::Display for ArgLenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect argument array length")
    }
}

impl error::Error for ArgLenMismatchError {
    fn description(&self) -> &str { "Incorrect argument array length" }
}

#[derive(Debug)]
pub struct ExecutionError;

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect execution result")
    }
}

impl error::Error for ExecutionError {
    fn description(&self) -> &str { "Incorrect execution result" }
}
