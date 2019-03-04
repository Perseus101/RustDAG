use std::hash::Hasher;
use std::marker::{Send, Sync};

use wasmi::{
    Error as InterpreterError, Trap, TrapKind, ModuleRef, Externals,
    RuntimeValue, RuntimeArgs, nan_preserving_float::{F32, F64}
};

use dag::contract::resolver::*;
use dag::contract::ContractValue;
use dag::storage::mpt::{
    MerklePatriciaTree, MPTStorageMap, NodeUpdates,
    node::Node, temp_map::MPTTempMap
};

use security::hash::hasher::Sha3Hasher;

pub trait ContractStateStorage = MPTStorageMap<ContractValue> + Send + Sync;

pub fn get_key(index: u32, contract: u64) -> u64 {
    let mut hasher = Sha3Hasher::new();
    hasher.write_u32(index);
    hasher.write_u64(contract);
    hasher.finish()
}

pub fn get_mapping_key(index: u32, key: u64, contract: u64) -> u64 {
    let mut hasher = Sha3Hasher::new();
    hasher.write_u32(index);
    hasher.write_u64(key);
    hasher.write_u64(contract);
    hasher.finish()
}
/// Cached state of a contract
///
/// Uses copy on write to only store updated state, and holds a reference to the
/// original contract state to access unmodified state.
pub struct CachedContractState<'a, M: ContractStateStorage> {
    module: &'a ModuleRef,
    state: MerklePatriciaTree<ContractValue, MPTTempMap<'a, ContractValue, M>>,
    contract: u64,
    root: u64,
}

impl<'a, M: ContractStateStorage> CachedContractState<'a, M> {

    /// Create a new cached state from a contract state
    pub fn new(module: &'a ModuleRef,
            state: MerklePatriciaTree<ContractValue, MPTTempMap<'a, ContractValue, M>>,
            contract: u64, root: u64) -> Self {
        CachedContractState {
            module,
            state,
            contract,
            root,
        }
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

    pub fn updates(self) -> Option<NodeUpdates<ContractValue>> {
        self.state.inner_map().write_out(&self.root).ok()
    }

    fn get_key(&self, index: u32) -> u64 {
        get_key(index, self.contract)
    }

    fn get_mapping_key(&self, index: u32, key: u64) -> u64 {
        get_mapping_key(index, key, self.contract)
    }

    fn get_u32(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.state.get(self.root, self.get_key(index)) {
            Ok(ContractValue::U32(val)) => Ok(Some(RuntimeValue::I32(*val as i32))),
            Ok(_) => Err(Trap::new(TrapKind::Unreachable)),
            Err(_) => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
        }
    }

    fn get_u64(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.state.get(self.root, self.get_key(index)) {
            Ok(ContractValue::U64(val)) => Ok(Some(RuntimeValue::I64(*val as i64))),
            Ok(_) => Err(Trap::new(TrapKind::Unreachable)),
            Err(_) => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
        }
    }

    fn get_f32(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.state.get(self.root, self.get_key(index)) {
            Ok(ContractValue::F32(val)) => Ok(Some(RuntimeValue::F32(F32::from(*val)))),
            Ok(_) => Err(Trap::new(TrapKind::Unreachable)),
            Err(_) => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
        }
    }

    fn get_f64(&self, index: u32) -> Result<Option<RuntimeValue>, Trap> {
        match self.state.get(self.root, self.get_key(index)) {
            Ok(ContractValue::F64(val)) => Ok(Some(RuntimeValue::F64(F64::from(*val)))),
            Ok(_) => Err(Trap::new(TrapKind::Unreachable)),
            Err(_) => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
        }
    }

    fn get_mapping(&self, index: u32, key: u64) -> Result<Option<RuntimeValue>, Trap> {
        match self.state.get(self.root, self.get_mapping_key(index, key)) {
            Ok(ContractValue::U64(val)) => Ok(Some(RuntimeValue::I64(*val as i64))),
            Ok(_) => Err(Trap::new(TrapKind::Unreachable)),
            Err(_) => Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
        }
    }

    fn set(&mut self, index: u64, value: ContractValue) {
        self.root = self.state.set(self.root, index, value);
    }

    fn set_u32(&mut self, index: u32, value: u32) -> Result<(), Trap> {
        let idx = self.get_key(index);
        self.set(idx, ContractValue::U32(value));
        Ok(())
    }

    fn set_u64(&mut self, index: u32, value: u64) -> Result<(), Trap> {
        let idx = self.get_key(index);
        self.set(idx, ContractValue::U64(value));
        Ok(())
    }

    fn set_f32(&mut self, index: u32, value: f32) -> Result<(), Trap> {
        let idx = self.get_key(index);
        self.set(idx, ContractValue::F32(value));
        Ok(())
    }

    fn set_f64(&mut self, index: u32, value: f64) -> Result<(), Trap> {
        let idx = self.get_key(index);
        self.set(idx, ContractValue::F64(value));
        Ok(())
    }

    fn set_mapping(&mut self, index: u32, key: u64, value: u64) -> Result<(), Trap> {
        let idx = self.get_mapping_key(index, key);
        self.set(idx, ContractValue::U64(value));
        Ok(())
    }
}

impl<'a, M: ContractStateStorage> Externals for CachedContractState<'a, M> {
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
    use std::collections::HashMap;

    use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder};

    use dag::storage::mpt::temp_map::MPTTempMap;

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
        let module = load_api_test_module_instance();
        let mut mpt = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        let contract_id = 0;

        // Update a single value
        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_u32",
                &[RuntimeValue::I32(0), RuntimeValue::I32(10)]).is_ok());

            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);

        // Assert the value is set
        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            // Error, out of bounds
            assert!(temp_state.exec("get_u32", &[RuntimeValue::I32(2)]).is_err());
            assert_eq!(Some(RuntimeValue::I32(10)), temp_state.exec("get_u32",
                &[RuntimeValue::I32(0)]).unwrap());

            // Error, out of bounds
            assert!(temp_state.exec("get_u32", &[RuntimeValue::I32(1)]).is_err());
        };

        // Update multiple values
        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_u32",
                &[RuntimeValue::I32(0), RuntimeValue::I32(15)]).is_ok());
            assert!(temp_state.exec("set_u32",
                &[RuntimeValue::I32(1), RuntimeValue::I32(100)]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);

        assert_eq!(mpt.get(root, get_key(0, 0)), Ok(&ContractValue::U32(15)));
        assert_eq!(mpt.get(root, get_key(1, 0)), Ok(&ContractValue::U32(100)));
        // Assert the values are changed
        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            // Error, out of bounds
            assert!(temp_state.exec("get_u32", &[RuntimeValue::I32(2)]).is_err());
            assert_eq!(Some(RuntimeValue::I32(15)), temp_state.exec("get_u32",
                &[RuntimeValue::I32(0)]).unwrap());
            assert_eq!(Some(RuntimeValue::I32(100)), temp_state.exec("get_u32",
                &[RuntimeValue::I32(1)]).unwrap());

            // Error, out of bounds
            assert!(temp_state.exec("get_u32", &[RuntimeValue::I32(3)]).is_err());
        };
    }

    #[test]
    fn test_api_resolver_u64() {
        let module = load_api_test_module_instance();
        let mut mpt = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        let contract_id = 0;

        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_u64",
                &[RuntimeValue::I32(0), RuntimeValue::I64(10)]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);

        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            // Error, out of bounds
            assert!(temp_state.exec("get_u64", &[RuntimeValue::I32(1)]).is_err());
            assert_eq!(Some(RuntimeValue::I64(10)), temp_state.exec("get_u64",
                &[RuntimeValue::I32(0)]).unwrap());
            // Error, out of bounds
            assert!(temp_state.exec("get_u64", &[RuntimeValue::I32(2)]).is_err());
        };
    }

    #[test]
    fn test_api_resolver_f32() {
        let module = load_api_test_module_instance();
        let mut mpt = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        let contract_id = 0;

        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_f32",
                &[RuntimeValue::I32(0), RuntimeValue::F32(10f32.into())]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);

        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            // Error, out of bounds
            assert!(temp_state.exec("get_f32", &[RuntimeValue::I32(1)]).is_err());
            assert_eq!(Some(RuntimeValue::F32(10f32.into())), temp_state.exec("get_f32",
                &[RuntimeValue::I32(0)]).unwrap());
            // Error, out of bounds
            assert!(temp_state.exec("get_f32", &[RuntimeValue::I32(2)]).is_err());
        };
    }

    #[test]
    fn test_api_resolver_f64() {
        let module = load_api_test_module_instance();
        let mut mpt = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        let contract_id = 0;

        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_f64",
                &[RuntimeValue::I32(0), RuntimeValue::F64(10f64.into())]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);

        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            // Error, out of bounds
            assert!(temp_state.exec("get_f64", &[RuntimeValue::I32(1)]).is_err());
            assert_eq!(Some(RuntimeValue::F64(10f64.into())), temp_state.exec("get_f64",
                &[RuntimeValue::I32(0)]).unwrap());
            // Error, out of bounds
            assert!(temp_state.exec("get_f64", &[RuntimeValue::I32(2)]).is_err());
        };
    }

    #[test]
    fn test_api_resolver_mapping() {
        let module = load_api_test_module_instance();
        let mut mpt = MerklePatriciaTree::new(HashMap::new());
        let mut root = mpt.default_root();
        let contract_id = 0;

        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("get_mapping",
                &[RuntimeValue::I32(0), RuntimeValue::I64(0)]).is_err());
            assert!(temp_state.exec("set_mapping",
                &[RuntimeValue::I32(0), RuntimeValue::I64(0), RuntimeValue::I64(0)]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);


        let updates = {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert!(temp_state.exec("set_mapping",
                &[RuntimeValue::I32(0), RuntimeValue::I64(1), RuntimeValue::I64(10)]).is_ok());
            temp_state.updates().unwrap()
        };

        root = updates.get_root_hash();
        mpt.commit_set(updates);
        assert_eq!(mpt.get(root, get_mapping_key(0, 0, contract_id)), Ok(&ContractValue::U64(0)));
        assert_eq!(mpt.get(root, get_mapping_key(0, 1, contract_id)), Ok(&ContractValue::U64(10)));

        {
            let mut temp_state = CachedContractState::new(&module,
                MerklePatriciaTree::new(MPTTempMap::new(&mpt)), contract_id, root);
            assert_eq!(Some(RuntimeValue::I64(0)), temp_state.exec("get_mapping",
                &[RuntimeValue::I32(0), RuntimeValue::I64(0)]).unwrap());
            assert_eq!(Some(RuntimeValue::I64(10)), temp_state.exec("get_mapping",
                &[RuntimeValue::I32(0), RuntimeValue::I64(1)]).unwrap());

            // Error, out of bounds
            assert!(temp_state.exec("get_mapping",
                &[RuntimeValue::I32(1), RuntimeValue::I64(0)]).is_err());
        };
    }
}