use dag::contract::Contract;

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Debug)]
pub enum TransactionData {
    Genesis,
    GenContract(Contract),
    ExecContract(()),
}
