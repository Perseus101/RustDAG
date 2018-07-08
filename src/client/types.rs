/// Stores the hashes returned from tip selection
#[derive(Serialize, Deserialize)]
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

/// Stores the success or failure of a remote process
#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessStatus {
    status: bool,
}

impl ProcessStatus {
    pub fn new(status: bool) -> ProcessStatus {
        ProcessStatus {
            status: status,
        }
    }
}