use std::error::Error;
use std::fmt;

use wasmi::HostError;
use wasmi::Error as WasmError;
use dag::storage::map::MapError;

#[derive(Debug)]
pub enum ContractError {
    //TODO: Consider refactoring so you can't have nested ContractError(WasmError(ContractError(WasmError(...))))
    WasmError(WasmError),
    RequiredFnNotFound,
    TypeMismatch,
    MapError(MapError)
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractError::WasmError(err) => write!(f, "Wasm Error: {}", err),
            ContractError::RequiredFnNotFound => write!(f, "Required function not found"),
            ContractError::TypeMismatch => write!(f, "Type mismatch"),
            ContractError::MapError(err) => write!(f, "Map Error: {}", err),
        }
    }
}

impl Error for ContractError {}
impl HostError for ContractError { }

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
