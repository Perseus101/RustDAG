use std::error::Error;
use std::fmt;

use wasmi::Error as WasmError;
use wasmi::HostError;

use dag::storage::map::MapError;

use super::ContractValue;

#[derive(Debug)]
pub enum ContractError {
    //TODO: Consider refactoring so you can't have nested ContractError(WasmError(ContractError(WasmError(...))))
    WasmError(WasmError),
    MapError(MapError),
    NoUpdates(Option<ContractValue>),
    RequiredFnNotFound,
    TypeMismatch,
}

impl PartialEq<ContractError> for ContractError {
    fn eq(&self, other: &ContractError) -> bool {
        match (self, other) {
            (ContractError::WasmError(_), ContractError::WasmError(_)) => true,
            (ContractError::MapError(se), ContractError::MapError(oe)) => se.eq(oe),
            (ContractError::NoUpdates(val0), ContractError::NoUpdates(val1)) => val0.eq(val1),
            (ContractError::RequiredFnNotFound, ContractError::RequiredFnNotFound) => true,
            (ContractError::TypeMismatch, ContractError::TypeMismatch) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractError::WasmError(err) => write!(f, "Wasm Error: {}", err),
            ContractError::MapError(err) => write!(f, "Map Error: {}", err),
            ContractError::NoUpdates(val) => write!(f, "No Updates: {:?}", val),
            ContractError::RequiredFnNotFound => write!(f, "Required function not found"),
            ContractError::TypeMismatch => write!(f, "Type mismatch"),
        }
    }
}

impl Error for ContractError {}
impl HostError for ContractError {}

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
