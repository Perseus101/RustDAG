use std::error;
use std::fmt;
use std::ops::{Index,IndexMut};

use dag::contract::state::ContractState;

use super::op::ContractOp;

/// A ContractFunction represents the executable code for a single smart
/// contract function.
///
/// # Examples
///
/// This function takes one argument and stores the cube of that argument in the
/// contract's first state variable.
///
/// ```
/// # use rustdag_lib::dag::contract::source::op::ContractOp;
/// # use rustdag_lib::dag::contract::source::function::ContractFunction;
/// # use rustdag_lib::dag::contract::state::ContractState;
/// let func = ContractFunction::new(vec![
///     ContractOp::Mul((0, 0, 1)),
///     ContractOp::Mul((0, 1, 2)),
/// ], 1, 1);
///
/// assert_eq!(func.exec(ContractState::new(1), vec![1]).unwrap()[0], 1);
/// assert_eq!(func.exec(ContractState::new(1), vec![2]).unwrap()[0], 8);
/// assert_eq!(func.exec(ContractState::new(1), vec![3]).unwrap()[0], 27);
/// ```
/// Because we cannot directly cube a number with these operators, it takes two
/// steps to cube the variable. In order to do this, we take the argument,
/// square it, store the result in a stack variable, then multiply that stack
/// variable by the argument and store it in the output. This means we need an
/// argument space of 1, stack space of 1, and output space of 1. Since
/// variables are mapped args -> stack -> state, this means position 0 is the
/// argument, position 1 is the stack variable, and position 2 is the
/// output variable.
///
/// The function's pseudocode looks like this:
/// ```python
/// def cube(x):
///     temp = x * x
///     return temp * x
/// ```
///
/// The first operator squares the argument and stores it in a stack variable,
/// and the second multiplies the stack variable by the input, and stores the
/// value in the output.
#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractFunction {
    ops: Vec<ContractOp>,
    argc: usize,
    stack_size: usize,
}

impl ContractFunction {
    pub fn new(ops: Vec<ContractOp>, argc: usize, stack_size: usize) -> Self {
        ContractFunction {
            ops,
            argc,
            stack_size
        }
    }


    /// Execute the function with args and state
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
    pub fn exec(&self, state: ContractState, args: Vec<u8>)
        -> Result<ContractState, ArgLenMismatchError> {
        if args.len() != self.argc {
            return Err(ArgLenMismatchError);
        }

        let mut mem = _MemMap::new(args, self.stack_size, state);

        for op in self.ops.iter() {
            match op {
                ContractOp::Add((lhi, rhi, desti)) => {
                    let lhs = mem[*lhi];
                    let rhs = mem[*rhi];
                    mem[*desti] = lhs + rhs;
                }
                ContractOp::Mul((lhi, rhi, desti)) => {
                    let lhs = mem[*lhi];
                    let rhs = mem[*rhi];
                    mem[*desti] = lhs * rhs;
                }
                ContractOp::AddConst((lhs, rhi, desti)) => {
                    let rhs = mem[*rhi];
                    mem[*desti] = lhs + rhs;
                }
                ContractOp::MulConst((lhs, rhi, desti)) => {
                    let rhs = mem[*rhi];
                    mem[*desti] = lhs * rhs;
                }
            }
        }
        Ok(mem.state())
    }
}

#[derive(Debug)]
pub struct ArgLenMismatchError;
impl fmt::Display for ArgLenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect argument array length")
    }
}

impl error::Error for ArgLenMismatchError {
    fn description(&self) -> &str { "Incorrect argument array length" }
}

struct _MemMap {
    argv: Vec<u8>,
    stack: Vec<u8>,
    state: ContractState,
}

impl _MemMap {
    fn new(argv: Vec<u8>, stack_size: usize, state: ContractState) -> Self {
        _MemMap {
            argv,
            stack: vec![0; stack_size],
            state,
        }
    }

    fn state(self) -> ContractState {
        self.state
    }
}

impl Index<usize> for _MemMap {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.argv.len() {
            &self.argv[index]
        }
        else if index < self.argv.len() + self.stack.len() {
            &self.stack[index - self.argv.len()]
        }
        else {
            &self.state[index - self.argv.len() - self.stack.len()]
        }
    }
}

impl IndexMut<usize> for _MemMap {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        if index < self.argv.len() {
            &mut self.argv[index]
        }
        else if index < self.argv.len() + self.stack.len() {
            &mut self.stack[index - self.argv.len()]
        }
        else {
            &mut self.state[index - self.argv.len() - self.stack.len()]
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use dag::contract::state::ContractState;
    use dag::contract::source::op::ContractOp;

    #[test]
    fn test_arg_len_mismatch() {
        let func = ContractFunction::new(vec![ContractOp::Add((0, 0, 0))], 1, 0);

        assert!(func.exec(ContractState::new(0), vec![]).is_err());
    }

    #[test]
    fn test_exec_add() {
        let func = ContractFunction::new(vec![ContractOp::Add((0, 1, 1))], 1, 0);

        // Test updating state
        let state = ContractState::new(1);
        if let Ok(state) = func.exec(state, vec![1]) {
            assert_eq!(state[0], 1);
        }
    }

    #[test]
    fn test_exec_add_const() {
        let func = ContractFunction::new(vec![ContractOp::AddConst((1, 0, 1))], 1, 0);

        let state = ContractState::new(1);
        if let Ok(state) = func.exec(state, vec![0]) {
            assert_eq!(state[0], 1);
        }
    }

    #[test]
    fn test_exec_mul() {
        let func = ContractFunction::new(vec![ContractOp::Mul((0, 1, 1))], 1, 0);

        let mut state = ContractState::new(1);
        state[0] = 1;
        if let Ok(state) = func.exec(state, vec![2]) {
            assert_eq!(state[0], 2);
        }
    }

    #[test]
    fn test_exec_mul_const() {
        let func = ContractFunction::new(vec![ContractOp::MulConst((0, 0, 1))], 1, 0);

        let mut state = ContractState::new(1);
        state[0] = 1;
        if let Ok(state) = func.exec(state, vec![1]) {
            assert_eq!(state[0], 0);
        }
    }

    #[test]
    fn test_exec_many() {
        let func = ContractFunction::new(vec![
            ContractOp::Mul((0, 0, 1)),
            ContractOp::Mul((0, 1, 2)),
            ContractOp::Add((0, 2, 3)),
            ContractOp::AddConst((5, 3, 4))
        ], 1, 3);

        // Test updating state
        let state = ContractState::new(1);
        if let Ok(state) = func.exec(state, vec![1]) {
            assert_eq!(state[0], 7);
        }
    }
}