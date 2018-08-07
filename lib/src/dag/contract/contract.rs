use std::error::Error;

use super::source::ContractSource;
use super::state::ContractState;

/// Encapsulates logic and state of a smart contract
///
/// The executable functions are stored in a
/// [ContractSource](source/struct.ContractSource.html) instance. When executed,
/// they are run against this struct's
/// [ContractState](state/struct.ContractState.html) instance, which represents
/// the state of all the contract's global variables.
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

    /// Executes the contract function at fn_idx using args as arguments
    ///
    /// # Errors
    ///
    /// * ArgLenMismatchError if the number of arguments is incorrect for the
    /// specified function
    ///
    /// # Examples
    /// ```
    /// # use rustdag_lib::dag::contract::source::ContractSource;
    /// # use rustdag_lib::dag::contract::source::op::ContractOp;
    /// # use rustdag_lib::dag::contract::source::function::ContractFunction;
    /// # use rustdag_lib::dag::contract::state::ContractState;
    /// # use rustdag_lib::dag::contract::Contract;
    /// let src = ContractSource::new(vec![
    ///     ContractFunction::new(vec![ContractOp::Add((0, 1, 1))], 1, 0),
    ///     ContractFunction::new(vec![ContractOp::AddConst((1, 0, 0))], 0, 0),
    ///     ContractFunction::new(vec![ContractOp::Mul((0, 1, 1))], 1, 0),
    ///     ContractFunction::new(vec![ContractOp::MulConst((2, 0, 0))], 0, 0)
    /// ]);
    /// let mut contract = Contract::new(src, ContractState::new(1));
    ///
    /// // Add 3 to the state, i.e. 0 + 3 = 3
    /// contract.exec(0, vec![3]);
    /// assert_eq!(contract.get_state()[0], 3);
    ///
    /// // Add constant 1 to the state, i.e. 3 + 1 = 4
    /// contract.exec(1, vec![]);
    /// assert_eq!(contract.get_state()[0], 4);
    ///
    /// // Multiply the state by 3, i.e. 4 * 3 = 12
    /// contract.exec(2, vec![3]);
    /// assert_eq!(contract.get_state()[0], 12);
    ///
    /// // Multiply the state by constant 2, i.e. 12 * 2 = 24
    /// contract.exec(3, vec![]);
    /// assert_eq!(contract.get_state()[0], 24);
    /// ```
    pub fn exec(&mut self, fn_idx: usize, args: Vec<u8>)
            -> Result<(), Box<Error>> {
        self.state = self.src.get_function(fn_idx).exec(self.state.clone(), args)?;
        Ok(())
    }

    pub fn get_state(&self) -> &ContractState {
        &self.state
    }
}