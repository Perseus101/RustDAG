use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use wasmi::RuntimeValue;

/// Contract state value
///
/// Represents a single state value in a contract
#[derive(Serialize, Deserialize, Clone)]
pub enum StateValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Mapping(HashMap<u64, u64>)
}

impl From<RuntimeValue> for StateValue {
    fn from(val: RuntimeValue) -> Self {
        match val {
            RuntimeValue::I32(val) => StateValue::U32(val as u32),
            RuntimeValue::I64(val) => StateValue::U64(val as u64),
            RuntimeValue::F32(val) => StateValue::F32(val.to_float()),
            RuntimeValue::F64(val) => StateValue::F64(val.to_float()),
        }
    }
}

/// Represents the state of a contract
#[derive(Serialize, Deserialize, Clone)]
pub struct ContractState {
    state: Vec<StateValue>,
}

impl ContractState {
    pub fn new(initial_state: Vec<StateValue>) -> Self {
        ContractState {
            state: initial_state
        }
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn into_state(self) -> Vec<StateValue> {
        self.state
    }
}

impl Index<usize> for ContractState {
    type Output = StateValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.state[index]
    }
}

impl IndexMut<usize> for ContractState {
    fn index_mut(&mut self, index: usize) -> &mut StateValue {
        &mut self.state[index]
    }
}
