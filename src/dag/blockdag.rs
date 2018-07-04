use std::collections::HashMap;

use dag::transaction::Transaction;

use security::hash::proof::proof_of_work;

const BASE_TRUNK_HASH: &'static str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const BASE_BRANCH_HASH: &'static str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001";

pub struct BlockDAG {
    transactions: HashMap<String, Transaction>,
    tips: HashMap<String, Transaction>,
}

impl Default for BlockDAG {
	fn default() -> Self {
		let mut dag = BlockDAG {
			transactions: HashMap::new(),
            tips: HashMap::new(),
		};
        dag.add_transaction(Transaction::new(BASE_TRUNK_HASH.to_string(),
            BASE_BRANCH_HASH.to_string(), vec![], 0, 0, 0, "0".to_string()));
        dag.add_transaction(Transaction::new(BASE_TRUNK_HASH.to_string(),
            BASE_BRANCH_HASH.to_string(), vec![], 0, 1, 0, "1".to_string()));

        dag
	}
}

impl BlockDAG {
    /// Add a transaction to the dag
    ///
    /// Calling this function inserts the new transaction into the list
    /// of active tips, and moves all transactions it references from
    /// list of active tips to the list of transactions.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        for hash in transaction.get_all_refs() {
            if let Some(t) = self.tips.remove(&hash) {
                self.transactions.insert(hash, t);
            }
        }
        self.tips.insert(transaction.get_hash(), transaction);
    }

    /// Create a new transaction
    ///
    /// Create a new transaction that references the transactions specified by
    /// trunk_hash and branch_hash
    pub fn create_transaction(&mut self, trunk_hash: String, branch_hash: String) -> Option<Transaction> {
        let mut nonce = None;
        if let Some(trunk) = self.tips.get(&trunk_hash).map_or(self.transactions.get(&trunk_hash), |x| Some(x)) {
            if let Some(branch) = self.tips.get(&branch_hash).map_or(self.transactions.get(&branch_hash), |x| Some(x)) {
                let trunk_nonce = trunk.get_nonce();
                let branch_nonce = branch.get_nonce();
                nonce = Some(proof_of_work(trunk_nonce, branch_nonce));
            }
        }
        match nonce {
            Some(nonce) => {
                let transaction = Transaction::create(branch_hash, trunk_hash, vec![], nonce, "0".to_string());
                self.add_transaction(transaction.clone());
                Some(transaction)
            }
            None => None
        }
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

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: &'static str = "F622E5DA4B02C80614D847C8DE22826B09CC3F76D6EE08047BE3383361406B8F4BE31FB8BDE423E02DCDC7355B0CA46BF13A2613D7000529BD24B8AC526FAADE";
    const BRANCH_HASH: &'static str = "E963AED7AE7C30EDF493556D2F1E6CBE8D4475D5B11741CCD7B594D92093D45A357A367C8D3BC5476A306B3EE1055FBAA1C62E99DE81F23D4BCE5BAC5EF570D5";

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
        let transaction = Transaction::create(TRUNK_HASH.to_string(), BRANCH_HASH.to_string(), vec![], 0, "1".to_string());
        dag.add_transaction(transaction.clone());
        let tips = dag.get_tips();
        assert_eq!(tips.len(), 1);
        assert_eq!(*tips[0], transaction);
    }

    #[test]
    fn test_create_transaction() {
        let mut dag = BlockDAG::default();

        let transaction = dag.create_transaction(BRANCH_HASH.to_string(), TRUNK_HASH.to_string()).unwrap();
        {
            let tips = dag.get_tips();
            assert_eq!(tips.len(), 1);
            assert_eq!(*tips[0], transaction);
        }
        assert_eq!(None, dag.create_transaction(TRUNK_HASH.to_string(), "".to_string()));
        assert_eq!(None, dag.create_transaction("".to_string(), BRANCH_HASH.to_string()));
        assert_eq!(None, dag.create_transaction("".to_string(), "".to_string()));
    }
}
