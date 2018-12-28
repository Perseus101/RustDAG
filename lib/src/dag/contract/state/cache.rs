use std::collections::HashMap;

use wasmi::{
    Error as InterpreterError, Trap, TrapKind, ModuleRef, Externals,
    RuntimeValue, RuntimeArgs, nan_preserving_float::{F32, F64}
};

use dag::contract::resolver::*;
use dag::contract::{ContractValue, error::ContractError};

use super::ContractState;
use super::persistent::PersistentCachedContractState;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum StateIndex {
    U32(u32),
    U64(u32),
    F32(u32),
    F64(u32),
    Mapping(u32, u64),
}

/// Cached state of a contract
///
/// Uses copy on write to only store updated state, and holds a reference to the
/// original contract state to access unmodified state.
pub struct CachedContractState<'a> {
    modified: HashMap<StateIndex, ContractValue>,
    module: &'a ModuleRef,
    state: &'a ContractState
}

impl<'a> CachedContractState<'a> {

    /// Create a new cached state from a contract state
    pub fn new(module: &'a ModuleRef, state: &'a ContractState) -> Self {
        CachedContractState {
            modified: HashMap::new(),
            module,
            state
        }
    }

    /// Move the modified values in a different data structure to drop the
    /// reference to the contract state
    pub fn persist(self) -> PersistentCachedContractState {
        PersistentCachedContractState::new(self.modified)
    }

    /// Take ownership of the modified values and a new reference to the
    /// contract state to resume execution
    pub fn resume(cache: PersistentCachedContractState, module: &'a ModuleRef, state: &'a ContractState)
            -> Self {
        CachedContractState {
            modified: cache.modified,
            module,
            state
        }
    }

    /// Call the state size functions that are expected to exist in the contract
    /// and return their results
    ///
    /// #Returns
    ///
    /// (int32_cap, int64_cap, float32_cap, float64_cap, mapping_cap)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the required functions do not exist
    pub fn get_state_sizes(&mut self) -> Result<(u32, u32, u32, u32, u32), ContractError> {
        let int32_cap: u32 = self.exec("__ofc__state_u32", &[])?
            .ok_or_else(|| { ContractError::RequiredFnNotFound })
            .map(|rt_val| { match rt_val { RuntimeValue::I32(val) => Some(val as u32), _ => None } })?
            .ok_or_else(|| { ContractError::TypeMismatch })?;

        let int64_cap: u32 = self.exec("__ofc__state_u64", &[])?
            .ok_or_else(|| { ContractError::RequiredFnNotFound })
            .map(|rt_val| { match rt_val { RuntimeValue::I32(val) => Some(val as u32), _ => None } })?
            .ok_or_else(|| { ContractError::TypeMismatch })?;

        let float32_cap: u32 = self.exec("__ofc__state_f32", &[])?
            .ok_or_else(|| { ContractError::RequiredFnNotFound })
            .map(|rt_val| { match rt_val { RuntimeValue::I32(val) => Some(val as u32), _ => None } })?
            .ok_or_else(|| { ContractError::TypeMismatch })?;

        let float64_cap: u32 = self.exec("__ofc__state_f64", &[])?
            .ok_or_else(|| { ContractError::RequiredFnNotFound })
            .map(|rt_val| { match rt_val { RuntimeValue::I32(val) => Some(val as u32), _ => None } })?
            .ok_or_else(|| { ContractError::TypeMismatch })?;

        let mapping_cap: u32 = self.exec("__ofc__state_mapping", &[])?
            .ok_or_else(|| { ContractError::RequiredFnNotFound })
            .map(|rt_val| { match rt_val { RuntimeValue::I32(val) => Some(val as u32), _ => None } })?
            .ok_or_else(|| { ContractError::TypeMismatch })?;

        Ok((int32_cap, int64_cap, float32_cap, float64_cap, mapping_cap))
    }

    /// Execute a contract function
    ///
    /// Executes the contract function with the name func_name with args as arguments
    ///
    /// #Errors
    ///
    /// Returns an error if:
    /// * the function does not exist
    /// * given arguments doesn't match to function signature,
    /// * trap occurred at the execution time,
    pub fn exec(&mut self, func_name: &str, args: &[RuntimeValue])
            -> Result<Option<RuntimeValue>, InterpreterError> {
        self.module.invoke_export(func_name, args, self)
    }

    fn get_u32(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.modified.get(&StateIndex::U32(index)) {
            Some(ContractValue::U32(val)) => Ok(Some(RuntimeValue::I32(*val as i32))),
            Some(_) => Err(Trap::new(TrapKind::Unreachable)),
            None => { // Value is not in cache
                match self.state.int32.get(index as usize) {
                    Some(val) => Ok(Some(RuntimeValue::I32(*val as i32))),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                }
            },
        }
    }

    fn get_u64(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.modified.get(&StateIndex::U64(index)) {
            Some(ContractValue::U64(val)) => Ok(Some(RuntimeValue::I64(*val as i64))),
            Some(_) => Err(Trap::new(TrapKind::Unreachable)),
            None => { // Value is not in cache
                match self.state.int64.get(index as usize) {
                    Some(val) => Ok(Some(RuntimeValue::I64(*val as i64))),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                }
            },
        }
    }

    fn get_f32(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.modified.get(&StateIndex::F32(index)) {
            Some(ContractValue::F32(val)) => Ok(Some(RuntimeValue::F32(F32::from(*val)))),
            Some(_) => Err(Trap::new(TrapKind::Unreachable)),
            None => { // Value is not in cache
                match self.state.float32.get(index as usize) {
                    Some(val) => Ok(Some(RuntimeValue::F32(F32::from(*val)))),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                }
            },
        }
    }

    fn get_f64(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.modified.get(&StateIndex::F64(index)) {
            Some(ContractValue::F64(val)) => Ok(Some(RuntimeValue::F64(F64::from(*val)))),
            Some(_) => Err(Trap::new(TrapKind::Unreachable)),
            None => { // Value is not in cache
                match self.state.float64.get(index as usize) {
                    Some(val) => Ok(Some(RuntimeValue::F64(F64::from(*val)))),
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                }
            },
        }
    }

    fn get_mapping(&self, index: u32, key: u64) -> Result<Option<RuntimeValue>, Trap> {
        match self.modified.get(&StateIndex::Mapping(index, key)) {
            Some(ContractValue::U64(val)) => Ok(Some(RuntimeValue::I64(*val as i64))),
            Some(_) => Err(Trap::new(TrapKind::Unreachable)),
            None => { // Value is not in cache
                match self.state.mappings.get(index as usize) {
                    Some(mapping) => {
                        match mapping.get(&key) {
                            Some(val) => Ok(Some(RuntimeValue::I64(*val as i64))),
                            None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                        }
                    },
                    None => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
                }
            },
        }
    }

    fn set_u32(&mut self, index: u32, value: u32) -> Result<(), Trap> {
        if index as usize >= self.state.int32.len() {
            return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
        }
        self.modified.insert(StateIndex::U32(index), ContractValue::U32(value));
        Ok(())
    }

    fn set_u64(&mut self, index: u32, value: u64) -> Result<(), Trap> {
        if index as usize >= self.state.int64.len() {
            return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
        }
        self.modified.insert(StateIndex::U64(index), ContractValue::U64(value));
        Ok(())
    }

    fn set_f32(&mut self, index: u32, value: f32) -> Result<(), Trap> {
        if index as usize >= self.state.float32.len() {
            return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
        }
        self.modified.insert(StateIndex::F32(index), ContractValue::F32(value));
        Ok(())
    }

    fn set_f64(&mut self, index: u32, value: f64) -> Result<(), Trap> {
        if index as usize >= self.state.float64.len() {
            return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
        }
        self.modified.insert(StateIndex::F64(index), ContractValue::F64(value));
        Ok(())
    }

    fn set_mapping(&mut self, index: u32, key: u64, value: u64) -> Result<(), Trap> {
        if index as usize >= self.state.mappings.len() {
            return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
        }
        self.modified.insert(StateIndex::Mapping(index, key), ContractValue::U64(value));
        Ok(())
    }
}

impl<'a> Externals for CachedContractState<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            GET_INT32_INDEX => {
                let index: u32 = args.nth(0);
                self.get_u32(index)
            },
            GET_INT64_INDEX => {
                let index: u32 = args.nth(0);
                self.get_u64(index)
            },
            GET_FLOAT32_INDEX => {
                let index: u32 = args.nth(0);
                self.get_f32(index)
            },
            GET_FLOAT64_INDEX => {
                let index: u32 = args.nth(0);
                self.get_f64(index)
            },
            GET_MAPPING_INDEX => {
                let index: u32 = args.nth(0);
                let key: u64 = args.nth(1);
                self.get_mapping(index, key)
            },


            SET_INT32_INDEX => {
                let index: u32 = args.nth(0);
                let value: u32 = args.nth(1);
                self.set_u32(index, value)?;
                Ok(None)
            },
            SET_INT64_INDEX => {
                let index: u32 = args.nth(0);
                let value: u64 = args.nth(1);
                self.set_u64(index, value)?;
                Ok(None)
            },
            SET_FLOAT32_INDEX => {
                let index: u32 = args.nth(0);
                let value: F32 = args.nth(1);
                self.set_f32(index, value.to_float())?;
                Ok(None)
            },
            SET_FLOAT64_INDEX => {
                let index: u32 = args.nth(0);
                let value: F64 = args.nth(1);
                self.set_f64(index, value.to_float())?;
                Ok(None)
            },
            SET_MAPPING_INDEX => {
                let index: u32 = args.nth(0);
                let key: u64 = args.nth(1);
                let value: u64 = args.nth(2);
                self.set_mapping(index, key, value)?;
                Ok(None)
            },

            _ => Err(Trap::new(TrapKind::Unreachable)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;

    use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder};

    use dag::contract::state::ContractState;

    fn load_module_from_file(filename: String) -> Module {
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");
        Module::from_buffer(&buf).expect("Could not parse file into WASM module")
    }

    fn load_api_test_module_instance() -> ModuleRef {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/raw_api_test.wasm");
        let module = load_module_from_file(d.to_str().unwrap().to_string());

        let mut imports = ImportsBuilder::new();
        imports.push_resolver("env", &Resolver);
        ModuleInstance::new(&module, &imports)
            .expect("Failed to instantiate module")
            .assert_no_start()
    }

    #[test]
    fn test_api_resolver_u32() {
        let state = ContractState::new(1, 0, 0, 0, 0);
        let module = load_api_test_module_instance();
        let mut temp_state = CachedContractState::new(&module, &state);

        assert_eq!(Some(RuntimeValue::I32(0)), temp_state.exec("get_u32",
            &[RuntimeValue::I32(0)]).unwrap());

        assert!(temp_state.exec("set_u32",
            &[RuntimeValue::I32(0), RuntimeValue::I32(10)]).is_ok());

        assert_eq!(Some(RuntimeValue::I32(10)), temp_state.exec("get_u32",
            &[RuntimeValue::I32(0)]).unwrap());

        // Error, out of bounds
        assert!(temp_state.exec("get_u32", &[RuntimeValue::I32(1)]).is_err());
    }

    #[test]
    fn test_api_resolver_u64() {
        let state = ContractState::new(0, 1, 0, 0, 0);
        let module = load_api_test_module_instance();
        let mut temp_state = CachedContractState::new(&module, &state);

        assert_eq!(Some(RuntimeValue::I64(0)), temp_state.exec("get_u64",
            &[RuntimeValue::I32(0)]).unwrap());

        assert!(temp_state.exec("set_u64",
            &[RuntimeValue::I32(0), RuntimeValue::I64(10)]).is_ok());

        assert_eq!(Some(RuntimeValue::I64(10)), temp_state.exec("get_u64",
            &[RuntimeValue::I32(0)]).unwrap());

        // Error, out of bounds
        assert!(temp_state.exec("get_u64", &[RuntimeValue::I32(1)]).is_err());
    }

    #[test]
    fn test_api_resolver_f32() {
        let state = ContractState::new(0, 0, 1, 0, 0);
        let module = load_api_test_module_instance();
        let mut temp_state = CachedContractState::new(&module, &state);

        assert_eq!(Some(RuntimeValue::F32(0f32.into())), temp_state.exec(
            "get_f32", &[RuntimeValue::I32(0)]).unwrap());

        assert!(temp_state.exec("set_f32",
            &[RuntimeValue::I32(0), RuntimeValue::F32(10f32.into())]).is_ok());

        assert_eq!(Some(RuntimeValue::F32(10f32.into())), temp_state.exec(
            "get_f32", &[RuntimeValue::I32(0)]).unwrap());

        // Error, out of bounds
        assert!(temp_state.exec("get_f32", &[RuntimeValue::I32(1)]).is_err());
    }

    #[test]
    fn test_api_resolver_f64() {
        let state = ContractState::new(0, 0, 0, 1, 0);
        let module = load_api_test_module_instance();
        let mut temp_state = CachedContractState::new(&module, &state);

        assert_eq!(Some(RuntimeValue::F64(0f64.into())), temp_state.exec(
            "get_f64", &[RuntimeValue::I32(0)]).unwrap());

        assert!(temp_state.exec("set_f64",
            &[RuntimeValue::I32(0), RuntimeValue::F64(10f64.into())]).is_ok());

        assert_eq!(Some(RuntimeValue::F64(10f64.into())), temp_state.exec(
            "get_f64", &[RuntimeValue::I32(0)]).unwrap());

        // Error, out of bounds
        assert!(temp_state.exec("get_f64", &[RuntimeValue::I32(1)]).is_err());
    }

    #[test]
    fn test_api_resolver_mapping() {
        let state = ContractState::new(0, 0, 0, 0, 1);
        let module = load_api_test_module_instance();
        let mut temp_state = CachedContractState::new(&module, &state);

        assert!(temp_state.exec("get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0)]).is_err());

        assert!(temp_state.exec("set_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0), RuntimeValue::I64(0)]).is_ok());
        assert!(temp_state.exec("set_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(1), RuntimeValue::I64(10)]).is_ok());

        assert_eq!(Some(RuntimeValue::I64(0)), temp_state.exec("get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(0)]).unwrap());
        assert_eq!(Some(RuntimeValue::I64(10)), temp_state.exec("get_mapping",
            &[RuntimeValue::I32(0), RuntimeValue::I64(1)]).unwrap());

        // Error, out of bounds
        assert!(temp_state.exec("get_mapping",
            &[RuntimeValue::I32(1), RuntimeValue::I64(0)]).is_err());
    }
}