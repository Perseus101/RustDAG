use std::error::Error;
use std::fmt;

use wasmi::Error as WasmError;

#[derive(Debug)]
pub enum ContractError {
    WasmError
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractError::WasmError => write!(f, "Wasm Error"),
        }
    }
}

impl Error for ContractError {}

impl From<WasmError> for ContractError {
    fn from(_error: WasmError) -> Self {
        ContractError::WasmError
    }
}
