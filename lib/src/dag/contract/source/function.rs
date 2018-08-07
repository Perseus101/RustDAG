use std::error;
use std::fmt;
use std::ops::{Index,IndexMut};

use dag::contract::state::ContractState;

use super::op::ContractOp;

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