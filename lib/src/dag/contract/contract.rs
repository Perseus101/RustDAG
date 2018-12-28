use wasmi::RuntimeValue;

use super::source::ContractSource;
use super::state::ContractState;
use super::error::ContractError;

/// Represents the values that can be passed to a contract
pub enum ContractValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl From<ContractValue> for RuntimeValue {
    fn from(val: ContractValue) -> Self {
        match val {
            ContractValue::U32(val) => RuntimeValue::I32(val as i32),
            ContractValue::U64(val) => RuntimeValue::I64(val as i64),
            ContractValue::F32(val) => RuntimeValue::F32(val.into()),
            ContractValue::F64(val) => RuntimeValue::F64(val.into()),
        }
    }
}

/// Encapsulates logic and state of a smart contract
///
/// The executable functions are stored in a
/// [ContractSource](source/struct.ContractSource.html) instance. When executed,
/// they are run against this struct's
/// [ContractState](state/struct.ContractState.html) instance, which represents
/// the state of all the contract's global variables.
#[derive(Serialize, Deserialize, Clone)]
pub struct Contract {
    /// Source of the contract
    src: ContractSource,
    /// Current state of the contract
    state: ContractState,
}

impl Contract {
    pub fn new(src: ContractSource) -> Result<Self, ContractError> {
        let state = ContractState::create(&src)?;
        Ok(Contract {
            state: state,
            src: src
        })
    }

    /// Executes the contract function
    pub fn exec(&self, func_name: &str, args: &[ContractValue]) -> Result<(), ContractError> {
        // TODO
        Ok(())
    }

    pub fn get_state(&self) -> &ContractState {
        &self.state
    }
}