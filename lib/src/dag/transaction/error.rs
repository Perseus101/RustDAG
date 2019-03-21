use std::error::Error;
use std::fmt;

use dag::storage::map::MapError;

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    Rejected(String)
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionError::Rejected(reason) => write!(f, "Rejected: {:?}", reason),
        }
    }
}

impl Error for TransactionError {}

impl From<MapError> for TransactionError {
    fn from(error: MapError) -> Self {
        TransactionError::Rejected(format!("{:?}", error))
    }
}