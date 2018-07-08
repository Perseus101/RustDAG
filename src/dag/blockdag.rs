use std::collections::HashMap;

use dag::transaction::Transaction;

use security::hash::proof::valid_proof;

const BASE_TRUNK_HASH: u64 = 0;
const BASE_BRANCH_HASH: u64 = 1;

pub struct BlockDAG {
    transactions: HashMap<u64, Transaction>,
    tips: HashMap<u64, Transaction>,
}

impl Default for BlockDAG {
	fn default() -> Self {
		let mut dag = BlockDAG {
			transactions: HashMap::new(),
            tips: HashMap::new(),
		};
        let genesis_1 = Transaction::new(BASE_TRUNK_HASH,
            BASE_BRANCH_HASH, vec![], 0, 0, 0);
        let genesis_2 = Transaction::new(BASE_TRUNK_HASH,
            BASE_BRANCH_HASH, vec![], 0, 1, 0);
        dag.tips.insert(genesis_1.get_hash(), genesis_1);
        dag.tips.insert(genesis_2.get_hash(), genesis_2);

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
    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
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
            self.tips.remove(&t.get_hash());
            self.transactions.insert(t.get_hash(), t);
        }
        self.tips.insert(transaction.get_hash(), transaction);
        true
    }

    /// Returns the transaction specified by hash
    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        fn _some_clone(trans: &Transaction) -> Option<Transaction> {
            Some(trans.clone())
        }
        self.tips.get(&hash).map_or(
            self.transactions.get(&hash).and_then(_some_clone), _some_clone
        )
    }

    /// Get tips of the dag
    ///
    /// This function returns all tips of the dag.
    /// Any transaction with no transactions referencing it
    /// is considered a tip.
    pub fn get_tips(&self) -> Vec<&Transaction> {
        self.tips.values().collect()
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
        assert_eq!(tips.len(), 2);

        if tips[0].get_nonce() == 0 { assert_eq!(tips[0].get_hash(), TRUNK_HASH); }
        else { assert_eq!(tips[0].get_hash(), BRANCH_HASH); }

        if tips[1].get_nonce() == 0 { assert_eq!(tips[1].get_hash(), TRUNK_HASH); }
        else { assert_eq!(tips[1].get_hash(), BRANCH_HASH); }
    }

    #[test]
    fn test_add_transaction() {
        let mut dag = BlockDAG::default();
        let mut key = PrivateKey::new(&SHA512_256);
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], 136516);
        transaction.sign(&mut key);
        assert!(transaction.verify());
        assert!(dag.add_transaction(transaction.clone()));

        {
            let tips = dag.get_tips();
            assert_eq!(tips.len(), 1);
            assert_eq!(*tips[0], transaction);
        }

        let bad_transaction = Transaction::create(10, BRANCH_HASH, vec![], 0);
        assert!(!dag.add_transaction(bad_transaction));
    }
}
