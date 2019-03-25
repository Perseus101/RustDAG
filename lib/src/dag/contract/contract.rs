#![allow(clippy::derive_hash_xor_eq)]

use std::hash::{Hash, Hasher};

use ordered_float::OrderedFloat;

use wasmi::{ModuleInstance, ModuleRef, RuntimeValue};

use dag::storage::mpt::{temp_map::MPTTempMap, MerklePatriciaTree, NodeUpdates};

use super::error::ContractError;
use super::resolver::get_imports_builder;
use super::source::ContractSource;
use super::state::{ContractState, ContractStateStorage};

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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Contract {
    /// Source of the contract
    src: ContractSource,
    id: u64,
}

impl Contract {
    pub fn new<'a, M: ContractStateStorage>(
        src: ContractSource,
        id: u64,
        storage: &'a MerklePatriciaTree<ContractValue, M>,
        root: u64,
    ) -> Result<(Self, NodeUpdates<ContractValue>), ContractError> {
        let contract = Contract { src, id };

        let (_, updates) = contract.exec("init", &Vec::new(), storage, root)?;

        Ok((contract, updates))
    }

    fn get_module(&self) -> Result<ModuleRef, ContractError> {
        let imports = get_imports_builder();
        Ok(ModuleInstance::new(&self.src.get_wasm_module()?, &imports)?.assert_no_start())
    }

    fn build_state<'a, M: ContractStateStorage>(
        &self,
        module: &'a ModuleRef,
        storage: &'a MerklePatriciaTree<ContractValue, M>,
        root: u64,
    ) -> Result<ContractState<'a, M>, ContractError> {
        Ok(ContractState::new(
            module,
            MerklePatriciaTree::new(MPTTempMap::new(storage)),
            self.id,
            root,
        ))
    }

    /// Execute the contract function
    pub fn exec<'a, M: ContractStateStorage>(
        &self,
        func_name: &str,
        args: &[ContractValue],
        storage: &'a MerklePatriciaTree<ContractValue, M>,
        root: u64,
    ) -> Result<(Option<ContractValue>, NodeUpdates<ContractValue>), ContractError> {
        let module = self.get_module()?;
        let mut temp_state = self.build_state(&module, storage, root)?;
        let return_value = self.exec_from_state(func_name, args, &mut temp_state)?;
        let updates = temp_state.updates()?;
        return Ok((return_value, updates));
    }

    /// Execute the contract function
    ///
    /// Ignores node updates and only returns the value returned by the function call
    pub fn exec_const<'a, M: ContractStateStorage>(
        &self,
        func_name: &str,
        args: &[ContractValue],
        storage: &'a MerklePatriciaTree<ContractValue, M>,
        root: u64,
    ) -> Result<Option<ContractValue>, ContractError> {
        let module = self.get_module()?;
        let mut temp_state = self.build_state(&module, storage, root)?;
        self.exec_from_state(func_name, args, &mut temp_state)
    }

    fn exec_from_state<M: ContractStateStorage>(
        &self,
        func_name: &str,
        args: &[ContractValue],
        state: &mut ContractState<M>,
    ) -> Result<Option<ContractValue>, ContractError> {
        let return_value = state
            .exec(
                func_name,
                &args
                    .iter()
                    .map(|x| RuntimeValue::from(x.clone()))
                    .collect::<Vec<_>>(),
            )?
            .map(|value| ContractValue::from(value));
        Ok(return_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    use dag::contract::state::{get_key, get_mapping_key};
    use dag::storage::map::OOB;

    #[test]
    fn test_exec_contract() {
        // Load the example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf)
            .expect("Could not read test file");

        let mut storage = MerklePatriciaTree::<ContractValue, _>::new(HashMap::new());
        let mut root = storage.default_root();
        let (contract, updates) = Contract::new(ContractSource::new(&buf), 0, &storage, root)
            .expect("Failed to create contract");
        root = updates.get_root_hash();
        assert!(storage.commit_set(updates).is_ok());

        let values = vec![
            ContractValue::U32(1),
            ContractValue::U64(2),
            ContractValue::F32(3f32),
            ContractValue::F64(4f64),
        ];
        let mapping_key = get_mapping_key(4, 0, 0);
        let mapping_val = ContractValue::U64(5);

        // Assert the values were set correctly
        for (i, val) in values.iter().enumerate() {
            assert_eq!(
                Ok(OOB::Borrowed(val)),
                storage.get(root, get_key(i as u32, 0))
            );
        }
        assert_eq!(
            Ok(OOB::Borrowed(&mapping_val)),
            storage.get(root, mapping_key)
        );

        // Now, assert the correct values also come out of WASM
        assert_eq!(
            Some(ContractValue::U32(1)),
            contract
                .exec_const("get_u32", &[ContractValue::U32(0)], &storage, root)
                .unwrap()
        );
        assert_eq!(
            Some(ContractValue::U64(2)),
            contract
                .exec_const("get_u64", &[ContractValue::U32(1)], &storage, root)
                .unwrap()
        );
        assert_eq!(
            Some(ContractValue::F32(3f32)),
            contract
                .exec_const("get_f32", &[ContractValue::U32(2)], &storage, root)
                .unwrap()
        );
        assert_eq!(
            Some(ContractValue::F64(4f64)),
            contract
                .exec_const("get_f64", &[ContractValue::U32(3)], &storage, root)
                .unwrap()
        );
        assert_eq!(
            Some(ContractValue::U64(5)),
            contract
                .exec_const(
                    "get_mapping",
                    &[ContractValue::U32(4), ContractValue::U64(0)],
                    &storage,
                    root
                )
                .unwrap()
        );
    }
}
