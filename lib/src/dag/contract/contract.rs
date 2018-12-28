use wasmi::{RuntimeValue, ModuleInstance};

use super::source::ContractSource;
use super::error::ContractError;
use super::resolver::get_imports_builder;
use super::state::{
    ContractState,
    cache::CachedContractState,
    persistent::PersistentCachedContractState
};

/// Represents the values that can be passed to a contract
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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

impl From<RuntimeValue> for ContractValue {
    fn from(val: RuntimeValue) -> Self {
        match val {
            RuntimeValue::I32(val) => ContractValue::U32(val as u32),
            RuntimeValue::I64(val) => ContractValue::U64(val as u64),
            RuntimeValue::F32(val) => ContractValue::F32(val.into()),
            RuntimeValue::F64(val) => ContractValue::F64(val.into()),
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
    pub fn exec(&self, func_name: &str, args: &[ContractValue])
            -> Result<(Option<ContractValue>, PersistentCachedContractState), ContractError> {
        let imports = get_imports_builder();
        let module = ModuleInstance::new(&self.src.get_wasm_module()?, &imports)?
            .assert_no_start();

        let temp_state = CachedContractState::new(&module, &self.state);
        self.exec_from_cached_state(func_name, args, temp_state)
    }

    /// Executes the contract function using persisted cached state
    pub fn exec_persisted(&self, func_name: &str, args: &[ContractValue], state: PersistentCachedContractState)
            -> Result<(Option<ContractValue>, PersistentCachedContractState), ContractError> {
        let imports = get_imports_builder();
        let module = ModuleInstance::new(&self.src.get_wasm_module()?, &imports)?
            .assert_no_start();

        let temp_state = CachedContractState::resume(state, &module, &self.state);
        self.exec_from_cached_state(func_name, args, temp_state)
    }

    pub fn get_state(&self) -> &ContractState {
        &self.state
    }

    pub fn writeback(&mut self, cache: PersistentCachedContractState) -> Result<(), ContractError> {
        cache.writeback(&mut self.state)
    }

    fn exec_from_cached_state(&self, func_name: &str, args: &[ContractValue], mut state: CachedContractState)
            -> Result<(Option<ContractValue>, PersistentCachedContractState), ContractError> {
        let return_value = state.exec(func_name, &args.into_iter()
            .map(|x| RuntimeValue::from(x.clone())).collect::<Vec<_>>())?
            .map(|value| { ContractValue::from(value) });
        Ok((return_value, state.persist()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_exec_contract() {
        // Load the example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/full_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        let contract = Contract::new(ContractSource::new(&buf)).expect("Failed to create contract");

        assert_eq!(Some(ContractValue::U32(1)),
            contract.exec("get_u32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(1)),
            contract.exec("get_u64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F32(1f32)),
            contract.exec("get_f32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F64(1f64)),
            contract.exec("get_f64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(1)),
            contract.exec("get_mapping", &[ContractValue::U32(0),
                ContractValue::U64(0)]).unwrap().0);
    }

    #[test]
    fn test_exec_persisted_contract() {
        // Load the example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/full_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        let mut contract = Contract::new(ContractSource::new(&buf)).expect("Failed to create contract");

        let (return_val, persisted) = contract.exec("set_u32",
            &[ContractValue::U32(0), ContractValue::U32(2)]).unwrap();
        assert!(return_val.is_none());

        let (return_val, persisted) = contract.exec_persisted("set_u64",
            &[ContractValue::U32(0), ContractValue::U64(2)], persisted).unwrap();
        assert!(return_val.is_none());

        let (return_val, persisted) = contract.exec_persisted("set_f32",
            &[ContractValue::U32(0), ContractValue::F32(2f32)], persisted).unwrap();
        assert!(return_val.is_none());

        let (return_val, persisted) = contract.exec_persisted("set_f64",
            &[ContractValue::U32(0), ContractValue::F64(2f64)], persisted).unwrap();
        assert!(return_val.is_none());

        let (return_val, persisted) = contract.exec_persisted("set_mapping",
            &[ContractValue::U32(0), ContractValue::U64(0), ContractValue::U64(2)], persisted).unwrap();
        assert!(return_val.is_none());

        // Updated state is still cached, old values should be unaffected
        assert_eq!(Some(ContractValue::U32(1)),
            contract.exec("get_u32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(1)),
            contract.exec("get_u64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F32(1f32)),
            contract.exec("get_f32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F64(1f64)),
            contract.exec("get_f64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(1)),
            contract.exec("get_mapping", &[ContractValue::U32(0),
                ContractValue::U64(0)]).unwrap().0);

        // Write back cache and assert values are changed
        contract.writeback(persisted).expect("Error while writing cached changes");
        assert_eq!(Some(ContractValue::U32(2)),
            contract.exec("get_u32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(2)),
            contract.exec("get_u64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F32(2f32)),
            contract.exec("get_f32", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::F64(2f64)),
            contract.exec("get_f64", &[ContractValue::U32(0)]).unwrap().0);
        assert_eq!(Some(ContractValue::U64(2)),
            contract.exec("get_mapping", &[ContractValue::U32(0),
                ContractValue::U64(0)]).unwrap().0);
    }
}