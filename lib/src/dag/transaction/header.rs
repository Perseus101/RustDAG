#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct TransactionHeader {
    pub(crate) branch_transaction: u64,
    pub(crate) trunk_transaction: u64,
    pub(crate) contract: u64,
    pub(crate) root: u64,
    pub(crate) timestamp: u64,
    pub(crate) nonce: u32,
}

impl TransactionHeader {
    pub fn new(
        branch_transaction: u64,
        trunk_transaction: u64,
        contract: u64,
        root: u64,
        timestamp: u64,
        nonce: u32,
    ) -> Self {
        TransactionHeader {
            branch_transaction,
            trunk_transaction,
            contract,
            root,
            timestamp,
            nonce,
        }
    }
}
