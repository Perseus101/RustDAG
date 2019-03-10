use std::error::Error;
use std::fmt;

use wasmi::Error as WasmError;
use dag::storage::map::MapError;

#[derive(Debug)]
pub enum ContractError {
    WasmError(WasmError),
    MapError(MapError),
    RequiredFnNotFound,
    TypeMismatch,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractError::WasmError(err) => write!(f, "Wasm Error: {:?}", err),
            ContractError::MapError(err) => write!(f, "Map Error: {:?}", err),
            ContractError::RequiredFnNotFound => write!(f, "Required function not found"),
            ContractError::TypeMismatch => write!(f, "Type mismatch"),
        }
    }
}

impl Error for ContractError {}

impl From<WasmError> for ContractError {
    fn from(error: WasmError) -> Self {
        ContractError::WasmError(error)
    }
}

impl From<MapError> for ContractError {
    fn from(error: MapError) -> Self {
        ContractError::MapError(error)
    }
}
