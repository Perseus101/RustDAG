use std::collections::HashMap;

use wasmi::{ModuleInstance, ImportsBuilder};

use dag::contract::{
    error::ContractError,
    source::ContractSource,
    resolver::Resolver
};

use super::cache::CachedContractState;

/// Represents the state of a contract
#[derive(Serialize, Deserialize, Clone)]
pub struct ContractState {
    pub(super) int32: Vec<u32>,
    pub(super) int64: Vec<u64>,
    pub(super) float32: Vec<f32>,
    pub(super) float64: Vec<f64>,
    pub(super) mappings: Vec<HashMap<u64, u64>>,
}

impl ContractState {

    /// Create a new contract state
    ///
    /// # Arguments
    ///
    /// * `int32_cap`: Capacity for u32 state values
    /// * `int64_cap`: Capacity for u64 state values
    /// * `float32_cap`: Capacity for f32 state values
    /// * `float64_cap`: Capacity for f64 state values
    /// * `mapping_cap`: Capacity for mapping state values
    pub fn new(int32_cap: u32, int64_cap: u32, float32_cap: u32,
            float64_cap: u32, mapping_cap: u32) -> Self {
        ContractState {
            int32: vec![0; int32_cap as usize],
            int64: vec![0; int64_cap as usize],
            float32: vec![0f32; float32_cap as usize],
            float64: vec![0f64; float64_cap as usize],
            mappings: vec![HashMap::new(); mapping_cap as usize],
        }
    }

    /// Create a new contract state
    ///
    /// Creates a new contract state from the contract source
    pub fn create(source: &ContractSource) -> Result<Self, ContractError> {
        let temp_state = ContractState::new(0, 0, 0, 0, 0);
        let module = source.get_wasm_module()?;
        let mut imports = ImportsBuilder::new();
        imports.push_resolver("env", &Resolver);
        let module_ref = ModuleInstance::new(&module, &imports)?.assert_no_start();

        // Get state sizes
        let persistent;
        let state_sizes;
        {
            let mut cache = CachedContractState::new(&module_ref, &temp_state);
            state_sizes = cache.get_state_sizes()?;
            persistent = cache.persist();
        }
        drop(temp_state);

        // Create state
        let mut state = ContractState::new(state_sizes.0, state_sizes.1,
            state_sizes.2, state_sizes.3, state_sizes.4);

        // Call the init function and apply the resulting state modifications
        let final_persistent;
        {
            let mut cache = CachedContractState::resume(persistent, &module_ref, &state);
            cache.exec("init", &[])?;
            final_persistent = cache.persist();
        }
        final_persistent.writeback(&mut state)?;

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_create_empty_state() {
        assert!(ContractState::create(&ContractSource::new(&[])).is_err());
    }

    #[test]
    fn test_create_invalid_state() {
        // Load the raw example contract file
        // This file is almost a valid contract, but lacks some methods required
        // for setup, and should therefore fail
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/raw_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        assert!(ContractState::create(&ContractSource::new(&buf)).is_err());
    }

    #[test]
    fn test_create_valid_state() {
        // Load the example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/full_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        let state = ContractState::create(&ContractSource::new(&buf)).unwrap();
        assert_eq!(1, state.int32.len());
        assert_eq!(1, state.int64.len());
        assert_eq!(1, state.float32.len());
        assert_eq!(1, state.float64.len());
        assert_eq!(1, state.mappings.len());

        assert_eq!(1, state.int32[0]);
        assert_eq!(1, state.int64[0]);
        assert_eq!(1f32, state.float32[0]);
        assert_eq!(1f64, state.float64[0]);
        assert_eq!(Some(&1), state.mappings[0].get(&0));
    }
}