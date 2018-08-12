#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub struct ContractResult {
    fn_idx: usize,
    args: Vec<u8>,
    result: Vec<u8>
}

impl ContractResult {
    pub fn new(fn_idx: usize, args: Vec<u8>, result: Vec<u8>) -> Self {
        ContractResult {
            fn_idx,
            args,
            result
        }
    }

    pub fn get_fn_idx(&self) -> usize {
        self.fn_idx
    }

    pub fn get_argc(&self) -> usize {
        self.args.len()
    }

    pub fn get_args(&self) -> &Vec<u8> {
        &self.args
    }

    pub fn get_result(&self) -> &Vec<u8> {
        &self.result
    }
}