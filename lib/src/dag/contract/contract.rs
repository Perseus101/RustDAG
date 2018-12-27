use super::source::ContractSource;
use super::state::ContractState;
use super::error::ContractError;

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

impl From<ContractSource> for Contract {
    fn from(src: ContractSource) -> Self {
        // TODO
        Contract {
            state: ContractState::new(vec![]),
            src: src
        }
    }
}

impl Contract {
    pub fn new(src: ContractSource, state: ContractState) -> Self {
        Contract {
            src,
            state
        }
    }

    /// Executes the contract function
    pub fn exec(&self, fn_ident: String, args: Vec<u8>) -> Result<(), ContractError> {
        let module = self.src.get_wasm_module()?;

        Ok(())
    }

    pub fn get_state(&self) -> &ContractState {
        &self.state
    }
}