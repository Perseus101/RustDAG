use std::collections::HashMap;

use rand::{Rng,thread_rng};

use dag::transaction::Transaction;

use security::hash::proof::valid_proof;

use util::types::TransactionHashes;

const BASE_TRUNK_HASH: u64 = 0;
const BASE_BRANCH_HASH: u64 = 1;

pub struct BlockDAG {
    transactions: HashMap<u64, Transaction>,
    pending_transactions: HashMap<u64, Transaction>,
    tips: Vec<u64>,
}

impl Default for BlockDAG {
	fn default() -> Self {
		let mut dag = BlockDAG {
			transactions: HashMap::new(),
            pending_transactions: HashMap::new(),
            tips: Vec::new(),
		};
        let genesis_1 = Transaction::new(BASE_TRUNK_HASH,
            BASE_BRANCH_HASH, vec![], 0, 0, 0);
        let genesis_2 = Transaction::new(BASE_TRUNK_HASH,
            BASE_BRANCH_HASH, vec![], 0, 1, 0);

        let genesis_1_hash = genesis_1.get_hash();
        let genesis_2_hash = genesis_2.get_hash();
        dag.pending_transactions.insert(genesis_1_hash, genesis_1);
        dag.pending_transactions.insert(genesis_2_hash, genesis_2);
        dag.tips.push(genesis_1_hash);
        dag.tips.push(genesis_2_hash);

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
    /// exist, the function will return false
    pub fn add_transaction(&mut self, transaction: &Transaction) -> bool {
        let mut referenced = Vec::with_capacity(2);
        if let Some(trunk) = self.get_transaction(transaction.get_trunk_hash()) {
            if let Some(branch) = self.get_transaction(transaction.get_branch_hash()) {
                if !valid_proof(trunk.get_nonce(), branch.get_nonce(), transaction.get_nonce()) {
                    return false;
                }
                referenced.push(trunk);
                referenced.push(branch);
            }
            else { return false; }
        }
        else { return false; }

        // Verify the transaction's signature
        if !transaction.verify() { return false; }

        for hash in transaction.get_ref_hashes() {
            if let Some(t) = self.get_transaction(hash) {
                referenced.push(t);
            }
            else { return false; }
        }

        for t in referenced {
            self.tips.remove_item(&t.get_hash());
        }
        let hash = transaction.get_hash();
        self.pending_transactions.insert(hash, transaction.clone());
        self.tips.push(hash);
        true
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
    const TRUNK_HASH: u64 = 18035271841456622039;
    const BRANCH_HASH: u64 = 475765571055499685;

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
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], 136516);
        transaction.sign(&mut key);
        assert!(transaction.verify());
        assert!(dag.add_transaction(&transaction));

        {
            let tips = dag.get_tips();
            assert_eq!(tips.trunk_hash, transaction.get_hash());
            assert_eq!(tips.branch_hash, transaction.get_branch_hash());
        }

        let bad_transaction = Transaction::create(10, BRANCH_HASH, vec![], 0);
        assert!(!dag.add_transaction(&bad_transaction));
    }
}
