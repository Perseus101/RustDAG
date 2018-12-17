use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct IncompleteChain {
    missing_hashes: Vec<u64>
}

impl IncompleteChain {
    pub fn new(missing_hashes: Vec<u64>) -> Self {
        IncompleteChain {
            missing_hashes
        }
    }
}

impl fmt::Display for IncompleteChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash Collision")
    }
}

impl Error for IncompleteChain {}