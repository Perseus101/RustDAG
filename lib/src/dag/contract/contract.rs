#![allow(clippy::derive_hash_xor_eq)]

use std::hash::{Hash,Hasher};

use ordered_float::OrderedFloat;

use wasmi::{RuntimeValue, ModuleInstance};

use dag::storage::mpt::{MerklePatriciaTree, temp_map::MPTTempMap};

use super::source::ContractSource;
use super::error::ContractError;
use super::resolver::get_imports_builder;
use super::state::cache::{CachedContractState, ContractStateStorage};

/// Represents the values that can be passed to a contract
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ContractValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl Hash for ContractValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ContractValue::U32(val) => val.hash(state),
            ContractValue::U64(val) => val.hash(state),
            ContractValue::F32(val) => OrderedFloat::from(*val).hash(state),
            ContractValue::F64(val) => OrderedFloat::from(*val).hash(state),
        }
    }
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
    id: u64,
}

impl Contract {
    pub fn new(src: ContractSource, id: u64) -> Result<Self, ContractError> {
        Ok(Contract {
            src,
            id
        })
    }

    /// Executes the contract function
    pub fn exec<'a, M: ContractStateStorage>(&self, func_name: &str, args: &[ContractValue],
            storage: &'a MerklePatriciaTree<ContractValue, M>, root: u64)
            -> Result<Option<ContractValue>, ContractError> {
        let imports = get_imports_builder();
        let module = ModuleInstance::new(&self.src.get_wasm_module()?, &imports)?
            .assert_no_start();

        let temp_state = CachedContractState::new(&module,
            MerklePatriciaTree::new(MPTTempMap::new(storage)), self.id, root);
        self.exec_from_cached_state(func_name, args, temp_state)
    }

    fn exec_from_cached_state<M: ContractStateStorage>(&self,
            func_name: &str, args: &[ContractValue], mut state: CachedContractState<M>)
            -> Result<Option<ContractValue>, ContractError> {
        let return_value = state.exec(func_name, &args.iter()
            .map(|x| RuntimeValue::from(x.clone())).collect::<Vec<_>>())?
            .map(|value| { ContractValue::from(value) });
        Ok(return_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;
    use std::collections::HashMap;

    use dag::contract::state::cache::{get_key, get_mapping_key};

    #[test]
    fn test_exec_contract() {
        // Load the example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/full_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        let contract = Contract::new(ContractSource::new(&buf), 0).expect("Failed to create contract");
        let mut storage = MerklePatriciaTree::<ContractValue, _>::new(HashMap::new());
        let mut root = storage.default_root();

        let values = vec![
            ContractValue::U32(1),
            ContractValue::U64(2),
            ContractValue::F32(3f32),
            ContractValue::F64(4f64),
        ];
        for (i, val) in values.iter().enumerate() {
            root = storage.set(root, get_key(i as u32, 0), val.clone());
        }

        let mapping_key = get_mapping_key(4, 0, 0);
        let mapping_val = ContractValue::U64(5);
        root = storage.set(root, mapping_key, mapping_val.clone());

        // Assert the values were set correctly
        for (i, val) in values.iter().enumerate() {
            assert_eq!(Ok(val), storage.get(root, get_key(i as u32, 0)));
        }
        assert_eq!(Ok(&mapping_val), storage.get(root, mapping_key));

        // Now, assert the correct values also come out of WASM
        assert_eq!(Some(ContractValue::U32(1)),
            contract.exec("get_u32", &[ContractValue::U32(0)], &storage, root).unwrap());
        assert_eq!(Some(ContractValue::U64(2)),
            contract.exec("get_u64", &[ContractValue::U32(1)], &storage, root).unwrap());
        assert_eq!(Some(ContractValue::F32(3f32)),
            contract.exec("get_f32", &[ContractValue::U32(2)], &storage, root).unwrap());
        assert_eq!(Some(ContractValue::F64(4f64)),
            contract.exec("get_f64", &[ContractValue::U32(3)], &storage, root).unwrap());
        assert_eq!(Some(ContractValue::U64(5)),
            contract.exec("get_mapping", &[ContractValue::U32(4),
                ContractValue::U64(0)], &storage, root).unwrap());
    }
}