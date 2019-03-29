use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BlockDAGError {
    MergeError,
    IncompleteChain(Vec<u64>),
}

impl fmt::Display for BlockDAGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockDAGError::MergeError => write!(f, "Merge Error"),
            BlockDAGError::IncompleteChain(missing) => {
                write!(f, "Incomplete chain, missing {:?}", missing)
            }
        }
    }
}

impl Error for BlockDAGError {}