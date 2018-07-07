/// Stores the hashes returned from tip selection
#[derive(Serialize, Deserialize)]
pub struct TransactionHashes {
    pub trunk_hash: String,
    pub branch_hash: String,
}

impl TransactionHashes {
    pub fn new(trunk_hash: String, branch_hash: String) -> TransactionHashes {
        TransactionHashes {
            trunk_hash: trunk_hash,
            branch_hash: branch_hash,
        }
    }
}

/// Stores the success or failure of a remote process
#[derive(Serialize, Deserialize)]
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