use wasmi::{Module, Error as WasmError};

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractSource {
    code: Vec<u8>
}

impl ContractSource {
    /// Create contract from raw wasm source
    pub fn new(code: &[u8]) -> Self {
        ContractSource {
            code: code.to_vec(),
        }
    }

    /// Create a wasm module from the contract source
    pub fn get_wasm_module(&self) -> Result<Module, WasmError> {
        Module::from_buffer(&self.code)
    }
}
