use rustdag_lib::dag::transaction::header::TransactionHeader;

pub struct MergeHeader {
    trunk_root: u64,
    branch_root: u64,
    merge_root: u64,
    ancestor_root: u64,
}

impl MergeHeader {
    pub fn new(trunk_root: u64, branch_root: u64, merge_root: u64, ancestor_root: u64) -> Self {
        MergeHeader {
            trunk_root,
            branch_root,
            merge_root,
            ancestor_root,
        }
    }

    pub fn into_transaction_header(
        self,
        branch_transaction: u64,
        trunk_transaction: u64,
        contract: u64,
        timestamp: u64,
        nonce: u32,
    ) -> TransactionHeader {
        TransactionHeader::new(
            branch_transaction,
            trunk_transaction,
            contract,
            self.trunk_root,
            self.branch_root,
            self.merge_root,
            self.ancestor_root,
            timestamp,
            nonce,
        )
    }
}
