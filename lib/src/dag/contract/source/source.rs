use super::function::ContractFunction;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractSource {
    functions: Vec<ContractFunction>,
    state_size: usize
}

impl ContractSource {
    pub fn new(functions: Vec<ContractFunction>, state_size: usize) -> Self {
        ContractSource {
            functions,
            state_size
        }
    }

    pub fn get_function(&self, idx: usize) -> &ContractFunction {
        &self.functions[idx]
    }

    pub fn get_state_size(&self) -> usize {
        self.state_size
    }
}
