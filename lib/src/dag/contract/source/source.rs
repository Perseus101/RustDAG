use super::function::ContractFunction;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractSource {
    functions: Vec<ContractFunction>,
}

impl ContractSource {
    pub fn new(functions: Vec<ContractFunction>) -> Self {
        ContractSource {
            functions
        }
    }

    pub fn get_function(&self, idx: usize) -> &ContractFunction {
        &self.functions[idx]
    }
}
