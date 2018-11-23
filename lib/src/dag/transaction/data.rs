use dag::contract::source::ContractSource;
use dag::contract::result::ContractResult;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub enum TransactionData {
    Genesis,
    GenContract(ContractSource),
    ExecContract(ContractResult),
}
