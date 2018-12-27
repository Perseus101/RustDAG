use dag::contract::source::ContractSource;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub enum TransactionData {
    Genesis,
    GenContract(ContractSource),
    // TODO
    ExecContract,
}
