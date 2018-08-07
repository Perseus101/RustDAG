use std::error::Error;

use super::source::ContractSource;
use super::state::ContractState;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct Contract {
    src: ContractSource,
    state: ContractState
}

impl Contract {
    pub fn new(src: ContractSource, state: ContractState) -> Self {
        Contract {
            src,
            state
        }
    }

    pub fn exec(&mut self, fn_idx: usize, args: Vec<u8>)
            -> Result<(), Box<Error>> {
        self.src.get_function(fn_idx).exec(self.state.clone(), args)?;
        Ok(())
    }
}