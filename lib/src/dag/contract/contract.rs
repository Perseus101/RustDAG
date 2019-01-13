use std::error::Error;

use super::source::ContractSource;
use super::state::ContractState;
use super::result::ContractResult;
use super::errors::{ArgLenMismatchError, ExecutionError};

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

impl From<ContractSource> for Contract {
    fn from(src: ContractSource) -> Self {
        Contract {
            state: ContractState::new(src.get_state_size()),
            src
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
    /// ], 1);
    /// let mut contract: Contract = From::from(src);
    ///
    /// // Add 3 to the state, i.e. 0 + 3 = 3
    /// let res = contract.exec(0, vec![3]);
    /// contract.apply(res.unwrap());
    /// assert_eq!(contract.get_state()[0], 3);
    ///
    /// // Add constant 1 to the state, i.e. 3 + 1 = 4
    /// let res = contract.exec(1, vec![]);
    /// contract.apply(res.unwrap());
    /// assert_eq!(contract.get_state()[0], 4);
    ///
    /// // Multiply the state by 3, i.e. 4 * 3 = 12
    /// let res = contract.exec(2, vec![3]);
    /// contract.apply(res.unwrap());
    /// assert_eq!(contract.get_state()[0], 12);
    ///
    /// // Multiply the state by constant 2, i.e. 12 * 2 = 24
    /// let res = contract.exec(3, vec![]);
    /// contract.apply(res.unwrap());
    /// assert_eq!(contract.get_state()[0], 24);
    /// ```
    pub fn exec(&self, fn_idx: usize, args: Vec<u8>)
            -> Result<ContractResult, Box<Error>> {
        let result = self.src.get_function(fn_idx)
            .exec(&self.state, &args)?;

        Ok(ContractResult::new(
            fn_idx,
            args,
            result
        ))
    }

    /// Apply a ContractResult to the state of the contract
    ///
    /// # Errors
    ///
    /// * ArgLenMismatchError if the number of arguments is incorrect for the
    /// specified function
    pub fn apply(&mut self, result: ContractResult)
            -> Result<(), Box<Error>> {
        let func = self.src.get_function(result.get_fn_idx());
        if func.get_argc() != result.get_argc() {
            return Err(From::from(ArgLenMismatchError));
        }

        if !func.verify(&self.state, &result) {
            return Err(From::from(ExecutionError))
        }

        self.state.as_mut_slice().copy_from_slice(
            &result.get_result()[(func.get_argc() + func.get_stack_size())..]
        );
        Ok(())
    }

    pub fn get_state(&self) -> &ContractState {
        &self.state
    }
}