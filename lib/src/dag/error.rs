use std::error::Error;
use std::fmt;

use dag::{storage::map::MapError, transaction::error::TransactionError};

#[derive(Debug, PartialEq)]
pub enum BlockDAGError {
    MergeError,
    MapError(MapError),
    TransactionError(TransactionError),
    IncompleteChain(Vec<u64>),
}

impl fmt::Display for BlockDAGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockDAGError::MergeError => write!(f, "Merge Error"),
            BlockDAGError::MapError(err) => write!(f, "Map Error: {}", err),
            BlockDAGError::TransactionError(err) => write!(f, "Transaction Error: {}", err),
            BlockDAGError::IncompleteChain(missing) => {
                write!(f, "Incomplete chain, missing {:?}", missing)
            }
        }
    }
}

impl Error for BlockDAGError {}

impl From<MapError> for BlockDAGError {
    fn from(error: MapError) -> Self {
        BlockDAGError::MapError(error)
    }
}

impl From<TransactionError> for BlockDAGError {
    fn from(error: TransactionError) -> Self {
        BlockDAGError::TransactionError(error)
    }
}
