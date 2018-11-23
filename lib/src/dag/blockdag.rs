use std::collections::HashMap;

use rand::{Rng,thread_rng};

use dag::transaction::Transaction;
use dag::milestone::Milestone;
use dag::milestone::pending::{
    PendingMilestone,
    MilestoneEvent,
    error::MilestoneError
};

use security::hash::proof::valid_proof;

use util::types::{TransactionHashes,TransactionStatus};

const GENESIS_HASH: u64 = 0;

const MILESTONE_NONCE_MIN: u32 = 100000;
const MILESTONE_NONCE_MAX: u32 = 200000;

pub struct BlockDAG {
    transactions: HashMap<u64, Transaction>,
    pending_transactions: HashMap<u64, Transaction>,
    milestones: Vec<Milestone>,
    pending_milestone: PendingMilestone,
    tips: Vec<u64>,
}

impl Default for BlockDAG {
	fn default() -> Self {
        let genesis_transaction = Transaction::new(GENESIS_HASH, GENESIS_HASH, vec![], 0, 0, 0);
        let genesis_milestone = Milestone::new(GENESIS_HASH, genesis_transaction.clone());

		let mut dag = BlockDAG {
			transactions: HashMap::new(),
            pending_transactions: HashMap::new(),
            milestones: Vec::new(),
            pending_milestone: PendingMilestone::Approved(genesis_milestone.clone()),
            tips: Vec::new(),
		};

        let genesis_transaction_hash = genesis_transaction.get_hash();
        let genesis_branch = Transaction::new(genesis_transaction_hash, genesis_transaction_hash, vec![], 0, 0, 0);
        let genesis_branch_hash = genesis_branch.get_hash();

        dag.transactions.insert(genesis_transaction_hash, genesis_transaction);
        dag.pending_transactions.insert(genesis_branch_hash, genesis_branch);
        dag.tips.push(genesis_transaction_hash);
        dag.tips.push(genesis_branch_hash);
        dag.milestones.push(genesis_milestone);

        dag
	}
}

impl BlockDAG {

    /// Add a transaction to the dag
    ///
    /// Calling this function inserts the new transaction into the list
    /// of active tips, and moves all transactions it references from
    /// list of active tips to the list of transactions.
    ///
    /// If the transaction is not valid, either because the proof of work
    /// is invalid, or because one of the referenced transactions does not
    /// exist, the function will return TransactionStatus::Rejected
    pub fn add_transaction(&mut self, transaction: &Transaction) -> TransactionStatus {
        let mut referenced = Vec::with_capacity(2);
        if let Some(trunk) = self.get_transaction(transaction.get_trunk_hash()) {
            if let Some(branch) = self.get_transaction(transaction.get_branch_hash()) {
                if !valid_proof(trunk.get_nonce(), branch.get_nonce(), transaction.get_nonce()) {
                    return TransactionStatus::Rejected;
                }
                referenced.push(trunk);
                referenced.push(branch);
            }
            else { return TransactionStatus::Rejected; }
        }
        else { return TransactionStatus::Rejected; }

        // Verify the transaction's signature
        if !transaction.verify() { return TransactionStatus::Rejected; }

        for hash in transaction.get_ref_hashes() {
            if let Some(t) = self.get_transaction(hash) {
                referenced.push(t);
            }
            else { return TransactionStatus::Rejected; }
        }

        for t in referenced {
            self.tips.remove_item(&t.get_hash());
        }
        let hash = transaction.get_hash();
        self.pending_transactions.insert(hash, transaction.clone());
        self.tips.push(hash);

        if transaction.get_nonce() > MILESTONE_NONCE_MIN &&
            transaction.get_nonce() < MILESTONE_NONCE_MAX {
            if self.create_milestone(transaction.clone()) {
                return TransactionStatus::Milestone;
            }
        }
        TransactionStatus::Pending
    }

    /// Check the validity of a milestone and add it as a pending milestone
    ///
    /// Walks backward on the graph searching for the previous milestone to
    /// ensure the new milestone references the previous one
    ///
    /// If the milestone is added, returns true
    fn create_milestone(&mut self, transaction: Transaction) -> bool {
        let prev_milestone = self.milestones[self.milestones.len() - 1].clone();
        if self.walk_search(&transaction, prev_milestone.get_hash(), prev_milestone.get_timestamp()) {
            let mut error: Option<MilestoneError> = None;
            self.pending_milestone = match self.pending_milestone.clone()
                .next(MilestoneEvent::New((prev_milestone.get_hash(), transaction))) {
                    Ok(pending) => pending,
                    Err(err) => {
                        let (pending, error_data) = err.convert();
                        error = Some(error_data);
                        pending
                    }
            };

            error.is_none()
        }
        else {
            false
        }
    }

    /// Add a confirmed milestone to the list of milestones
    ///
    /// Walks backward on the graph searching for the previous milestone to
    /// ensure the new milestone references the previous one
    ///
    /// If the milestone is added, returns true
    fn add_milestone(&mut self, transaction: Transaction) -> bool {
        let prev_milestone = self.milestones[self.milestones.len() - 1].clone();
        if self.walk_search(&transaction, prev_milestone.get_hash(), prev_milestone.get_timestamp()) {
            self.confirm_transactions(&transaction);
            self.milestones.push(Milestone::new(prev_milestone.get_hash(), transaction));
            true
        }
        else {
            false
        }
    }

    /// Walk backwards from transaction, searching for a transaction specified
    /// by hash. Stops at any transaction that occurred before timestamp
    ///
    /// If the transaction is found, returns true
    fn walk_search(&self, transaction: &Transaction, hash: u64, timestamp: u64) -> bool {
        if transaction.get_timestamp() < timestamp {
            return false;
        }
        for transaction_hash in transaction.get_all_refs() {
            if let Some(transaction) = self.get_transaction(transaction_hash) {
                if transaction_hash == hash {
                    return true;
                }
                if self.walk_search(&transaction, hash, timestamp) {
                    return true;
                }
            }
        }
        false
    }

    /// Move all transactions referenced by transaction from
    /// pending_transactions to transactions
    fn confirm_transactions(&mut self, transaction: &Transaction) {
        for transaction_hash in transaction.get_all_refs() {
            if let Some(transaction) = self.pending_transactions.remove(&transaction_hash) {
                self.confirm_transactions(&transaction);
                self.transactions.insert(transaction_hash, transaction);
            }
        }
    }

    /// Returns the transaction specified by hash
    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        fn _some_clone(trans: &Transaction) -> Option<Transaction> {
            Some(trans.clone())
        }
        self.pending_transactions.get(&hash).map_or(
            self.transactions.get(&hash).and_then(_some_clone), _some_clone
        )
    }

    /// Get the confirmation status of a transaction specified by hash
    pub fn get_confirmation_status(&self, hash: u64) -> TransactionStatus {
        if let Some(_) = self.pending_transactions.get(&hash) {
            return TransactionStatus::Pending;
        }
        if let Some(_) = self.transactions.get(&hash) {
            return TransactionStatus::Accepted;
        }
        TransactionStatus::Rejected
    }

    /// Select tips from the dag
    ///
    /// This function will select 2 tips from the dag to use for a new
    /// transaction. Any transaction with no transactions referencing it is
    /// considered a tip.
    pub fn get_tips(&self) -> TransactionHashes {
        let trunk_tip;
        let branch_tip;
        if self.tips.len() > 1 {
            // Randomly select two unique transactions from the tips
            let mut rng = thread_rng();
            let trunk_tip_idx = rng.gen_range(0, self.tips.len());
            let mut branch_tip_idx = rng.gen_range(0, self.tips.len());
            while branch_tip_idx == trunk_tip_idx {
                branch_tip_idx = rng.gen_range(0, self.tips.len());
            }

            trunk_tip = self.tips[trunk_tip_idx];
            branch_tip = self.tips[branch_tip_idx];
        }
        else {
            trunk_tip = self.tips[0];
            branch_tip = self.get_transaction(trunk_tip).unwrap().get_branch_hash();
        }

        TransactionHashes::new(trunk_tip, branch_tip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dag::transaction::Transaction;

    use security::keys::PrivateKey;
    use security::ring::digest::SHA512_256;

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: u64 = 9160162714596186031;
    const BRANCH_HASH: u64 = 6508967370193414217;

    const BASE_NONCE: u32 = 18722;

    #[test]
    fn test_genesis_transactions() {
        let dag = BlockDAG::default();
        let tips = dag.get_tips();

        if tips.trunk_hash == TRUNK_HASH {
            assert_eq!(tips.branch_hash, BRANCH_HASH);
        }
        else {
            assert_eq!(tips.trunk_hash, BRANCH_HASH);
            assert_eq!(tips.branch_hash, TRUNK_HASH);
        }
    }

    #[test]
    fn test_add_transaction() {
        let mut dag = BlockDAG::default();
        let mut key = PrivateKey::new(&SHA512_256);
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], BASE_NONCE);
        transaction.sign(&mut key);
        assert!(transaction.verify());
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);

        {
            let tips = dag.get_tips();
            assert_eq!(tips.trunk_hash, transaction.get_hash());
            assert_eq!(tips.branch_hash, transaction.get_branch_hash());
        }

        let bad_transaction = Transaction::create(10, BRANCH_HASH, vec![], 0);
        assert_eq!(dag.add_transaction(&bad_transaction), TransactionStatus::Rejected);
    }

    #[test]
    fn test_walk_search() {
        let dag = BlockDAG::default();
        let prev_milestone = dag.milestones[dag.milestones.len() - 1].clone();

        let transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], 0);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0));

        let transaction = Transaction::create(TRUNK_HASH, 0, vec![], 0);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0));

        let transaction = Transaction::create(0, BRANCH_HASH, vec![], 0);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0));

        let transaction = Transaction::create(0, 0, vec![], 0);
        assert!(!dag.walk_search(&transaction, prev_milestone.get_hash(), 0));
    }

    #[test]
    fn test_add_milestone() {
        let mut dag = BlockDAG::default();
        let mut key = PrivateKey::new(&SHA512_256);
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], BASE_NONCE);
        transaction.sign(&mut key);
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
        assert!(dag.add_milestone(transaction));
    }

    #[test]
    fn test_get_confirmation_status() {
        let mut dag = BlockDAG::default();
        assert_eq!(dag.get_confirmation_status(TRUNK_HASH), TransactionStatus::Accepted);
        assert_eq!(dag.get_confirmation_status(BRANCH_HASH), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(10), TransactionStatus::Rejected);

        let mut key = PrivateKey::new(&SHA512_256);
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], BASE_NONCE);
        transaction.sign(&mut key);
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(transaction.get_hash()), TransactionStatus::Pending);

    }
}
