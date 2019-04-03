use std::error::Error;
use std::fmt;

use rustdag_lib::{dag::error::BlockDAGError, util::peer::Error as NetworkError};

#[derive(Debug)]
pub enum CliError {
    NetworkError(NetworkError),
    DagError(BlockDAGError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::NetworkError(err) => write!(f, "Network Error: {:?}", err),
            CliError::DagError(err) => write!(f, "Dag Error: {:?}", err),
        }
    }
}

impl Error for CliError {}

impl From<NetworkError> for CliError {
    fn from(error: NetworkError) -> Self {
        CliError::NetworkError(error)
    }
}

impl From<BlockDAGError> for CliError {
    fn from(error: BlockDAGError) -> Self {
        CliError::DagError(error)
    }
}
