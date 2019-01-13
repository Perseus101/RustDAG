use std::collections::HashMap;

use super::state::ContractState;
use super::cache::StateIndex;

use dag::contract::{ContractValue, error::ContractError};

/// Persistent cached state of a contract
///
/// This struct is used to persist a cached contract state without the reference
/// to the original contract state
#[derive(Clone)]
pub struct PersistentCachedContractState {
    pub(super) modified: HashMap<StateIndex, ContractValue>
}

impl PersistentCachedContractState {
    pub fn new(modified: HashMap<StateIndex, ContractValue>) -> Self {
        PersistentCachedContractState {
            modified
        }
    }

    /// Write temporary changes permanetly into state
    pub fn writeback(&self, state: &mut ContractState) -> Result<(), ContractError> {
        for (index, value) in self.modified.iter() {
            match index {
                StateIndex::U32(index) => {
                    match value {
                        ContractValue::U32(value) => state.int32[*index as usize] = *value,
                        _ => return Err(ContractError::TypeMismatch),
                    }
                }
                StateIndex::U64(index) => {
                    match value {
                        ContractValue::U64(value) => state.int64[*index as usize] = *value,
                        _ => return Err(ContractError::TypeMismatch),
                    }
                }
                StateIndex::F32(index) => {
                    match value {
                        ContractValue::F32(value) => state.float32[*index as usize] = *value,
                        _ => return Err(ContractError::TypeMismatch),
                    }
                }
                StateIndex::F64(index) => {
                    match value {
                        ContractValue::F64(value) => state.float64[*index as usize] = *value,
                        _ => return Err(ContractError::TypeMismatch),
                    }
                }
                StateIndex::Mapping(index, key) => {
                    match value {
                        ContractValue::U64(value) => { state.mappings[*index as usize].insert(*key, *value); },
                        _ => return Err(ContractError::TypeMismatch),
                    }
                }
            }
        }
        Ok(())
    }
}