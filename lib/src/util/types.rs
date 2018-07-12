/// Stores the hashes returned from tip selection
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionHashes {
    pub trunk_hash: u64,
    pub branch_hash: u64,
}

impl TransactionHashes {
    pub fn new(trunk_hash: u64, branch_hash: u64) -> TransactionHashes {
        TransactionHashes {
            trunk_hash: trunk_hash,
            branch_hash: branch_hash,
        }
    }
}

/// Stores the status of adding a transaction
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TransactionStatus {
    Accepted,
    Rejected,
    Pending,
    Milestone
}
