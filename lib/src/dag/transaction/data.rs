use dag::contract::{source::ContractSource, ContractValue};

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub enum TransactionData {
    Genesis,
    GenContract(ContractSource),
    ExecContract(String, Vec<ContractValue>),
    Empty,
}
