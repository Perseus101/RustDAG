use std::error::Error;
use std::fmt;

use dag::contract::error::ContractError;

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    Rejected(String),
    Contract(ContractError),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionError::Rejected(reason) => write!(f, "Rejected: {:?}", reason),
            TransactionError::Contract(err) => write!(f, "Contract Error: {}", err),
        }
    }
}

impl Error for TransactionError {}

impl From<ContractError> for TransactionError {
    fn from(error: ContractError) -> Self {
        TransactionError::Contract(error)
    }
}
