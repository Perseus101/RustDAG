use std::collections::HashMap;

use rand::{Rng,thread_rng};
use replace_with::replace_with_or_abort;

use dag::transaction::{Transaction, data::TransactionData};
use dag::contract::Contract;
use dag::milestone::Milestone;
use dag::milestone::pending::{
    PendingMilestone,
    MilestoneEvent,
    MilestoneSignature,
    error::MilestoneError
};

use super::incomplete_chain::IncompleteChain;

use security::hash::proof::valid_proof;

use util::types::{TransactionHashes,TransactionStatus};

const GENESIS_HASH: u64 = 0;

const MILESTONE_NONCE_MIN: u32 = 100000;
const MILESTONE_NONCE_MAX: u32 = 200000;

pub struct BlockDAG {
    transactions: HashMap<u64, Transaction>,
    pending_transactions: HashMap<u64, Transaction>,
    contracts: HashMap<u64, Contract>,
    milestones: Vec<Milestone>,
    pending_milestone: PendingMilestone,
    tips: Vec<u64>,
}

impl Default for BlockDAG {
	fn default() -> Self {
        let genesis_transaction = Transaction::new(GENESIS_HASH, GENESIS_HASH,
            vec![], 0, 0, 0, TransactionData::Genesis);
        let genesis_milestone = Milestone::new(GENESIS_HASH, genesis_transaction.clone());

		let mut dag = BlockDAG {
			transactions: HashMap::new(),
            pending_transactions: HashMap::new(),
            contracts: HashMap::new(),
            milestones: Vec::new(),
            pending_milestone: PendingMilestone::Approved(genesis_milestone.clone()),
            tips: Vec::new(),
		};

        let genesis_transaction_hash = genesis_transaction.get_hash();
        let genesis_branch = Transaction::new(genesis_transaction_hash,
            genesis_transaction_hash, vec![], 0, 0, 0, TransactionData::Genesis);
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

        let hash = transaction.get_hash();

        match transaction.get_data() {
            TransactionData::Genesis => {
                return TransactionStatus::Rejected;
            },
            TransactionData::GenContract(src) => {
                // Generate a new contract
                self.contracts.insert(hash, From::from(src.clone()));
            },
            TransactionData::ExecContract(result) => {
                // Execute the function on the contract
                if let Some(contract) = self.contracts.get_mut(&transaction.get_contract()) {
                    if let Err(_) = contract.apply(result.clone()) {
                        return TransactionStatus::Rejected;
                    }
                }
                else { return TransactionStatus::Rejected; }
            }
        }

        for hash in transaction.get_ref_hashes() {
            if let Some(t) = self.get_transaction(hash) {
                referenced.push(t);
            }
            else { return TransactionStatus::Rejected; }
        }

        for t in referenced {
            self.tips.remove_item(&t.get_hash());
        }
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

    /// Take the current pending milestone and apply transform_fn to it
    fn transform_pending_milestone<F>(&mut self, transform_fn: F)
            where F: FnOnce(PendingMilestone) -> PendingMilestone {
        replace_with_or_abort(&mut self.pending_milestone, transform_fn);
        let mut maybe_milestone: Option<Milestone> = None;
        if let PendingMilestone::Approved(milestone) = &self.pending_milestone {
            maybe_milestone = Some(milestone.clone());
        }

        if let Some(milestone) = maybe_milestone {
            self.confirm_transactions(milestone.get_transaction());
            self.milestones.push(milestone);
        }
    }

    /// Update the current pending milestone with event
    fn update_pending_milestone(&mut self, event: MilestoneEvent)
            -> Result<(), MilestoneError> {
        let mut result = Ok(());
        {
            let error = &mut result;
            self.transform_pending_milestone(move |pending_milestone| {
                match pending_milestone.next(event) {
                    Ok(pending) => pending,
                    Err(err) => {
                        let (pending, error_data) = err.convert();
                        *error = Err(error_data);
                        pending
                    }
                }
            });
        }
        result
    }

    /// Check the validity of a milestone and add it as a pending milestone
    ///
    /// Walks backward on the graph searching for the previous milestone to
    /// ensure the new milestone references the previous one
    ///
    /// If the milestone is added, returns true
    fn create_milestone(&mut self, transaction: Transaction) -> bool {
        let hash;
        // TODO This scope should not be required with NLL
        { hash = self.milestones[self.milestones.len() - 1].get_hash(); }
        let event = MilestoneEvent::New((hash, transaction));
        self.update_pending_milestone(event).is_ok()
    }

    /// Add a confirmed milestone to the list of milestones
    ///
    /// Walks backward on the graph searching for the previous milestone
    ///
    /// If the milestone is found, return the chain of transactions to it
    ///
    /// If the milestone is not found, return and IncompleteChain error with any
    /// transactions that were not found locally
    pub fn verify_milestone(&self, transaction: Transaction)
            -> Result<Vec<Transaction>, IncompleteChain> {
        let prev_milestone = &self.milestones[self.milestones.len() - 1];
        let mut transaction_chain: Vec<Transaction> = Vec::new();
        let mut missing_hashes: Vec<u64> = Vec::new();

        if self.walk_search(&transaction, prev_milestone.get_hash(), prev_milestone.get_timestamp(),
                &mut |transaction: &Transaction| {
                    transaction_chain.push(transaction.clone());
                },
                &mut |hash: u64| {
                    missing_hashes.push(hash);
                }) {
            Ok(transaction_chain)
        }
        else {
            Err(IncompleteChain::new(missing_hashes))
        }
    }

    /// Take a chain of milestones from the pending milestone to the previous
    /// milestone and pass them to the pending milestone state machine for
    /// confirmation
    pub fn process_chain(&mut self, chain: Vec<Transaction>) -> bool {
        for transaction in chain.into_iter() {
            if let Err(err) = self.update_pending_milestone(MilestoneEvent::Chain(transaction)) {
                // TODO log error
                println!("[ERROR] - process_chain - {:?}", err);
                return false;
            }
        }
        true
    }

    /// Add a signature to the current pending milestone
    pub fn add_pending_signature(&mut self, signature: MilestoneSignature) -> bool {
        self.update_pending_milestone(MilestoneEvent::Sign(signature)).is_ok()
    }

    /// Walk backwards from transaction, searching for a transaction specified
    /// by hash. Stops at any transaction that occurred before timestamp
    ///
    /// If the transaction is found, returns true
    fn walk_search<F, G>(&self, transaction: &Transaction, hash: u64,
            timestamp: u64, chain_function: &mut F, not_found_function: &mut G) -> bool
            where F: FnMut(&Transaction), G: FnMut(u64) {
        if transaction.get_timestamp() < timestamp {
            return false;
        }
        for transaction_hash in transaction.get_all_refs() {
            if let Some(transaction) = self.get_transaction(transaction_hash) {
                if transaction_hash == hash {
                    // This is the transaction we are looking for, return
                    return true;
                }
                if self.walk_search(&transaction, hash, timestamp, chain_function, not_found_function) {
                    // Found the transaction somewhere along this chain
                    chain_function(&transaction);
                    return true;
                }
            }
            else {
                not_found_function(transaction_hash);
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

    // Get the hash id's of all the contracts stored on the dag
    pub fn get_contracts(&self) -> Vec<u64> {
        self.contracts.keys().map(|x| *x).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dag::transaction::Transaction;
    use dag::contract::source::ContractSource;

    use security::keys::PrivateKey;
    use security::ring::digest::SHA512_256;

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: u64 = 18437817136469695293;
    const BRANCH_HASH: u64 = 6043537212972274484;

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
        let data = TransactionData::GenContract(ContractSource::new(vec![], 0));
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH,
            vec![], 0, BASE_NONCE, data);
        transaction.sign(&mut key);
        assert!(transaction.verify());
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);

        let tips = dag.get_tips();
        assert_eq!(tips.trunk_hash, transaction.get_hash());
        assert_eq!(tips.branch_hash, transaction.get_branch_hash());
        drop(tips);

        let bad_transaction = Transaction::create(10, BRANCH_HASH, vec![], 0, 0,
            TransactionData::Genesis);
        assert_eq!(dag.add_transaction(&bad_transaction), TransactionStatus::Rejected);
    }

    #[test]
    fn test_walk_search() {
        let dag = BlockDAG::default();
        let prev_milestone = dag.milestones[dag.milestones.len() - 1].clone();

        let transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![],
            0, 0, TransactionData::Genesis);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0, &mut |_| {}, &mut |_| {}));

        let transaction = Transaction::create(TRUNK_HASH, 0, vec![], 0, 0,
            TransactionData::Genesis);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0, &mut |_| {}, &mut |_| {}));

        let transaction = Transaction::create(0, BRANCH_HASH, vec![], 0, 0,
            TransactionData::Genesis);
        assert!(dag.walk_search(&transaction, prev_milestone.get_hash(), 0, &mut |_| {}, &mut |_| {}));

        let transaction = Transaction::create(0, 0, vec![], 0, 0,
            TransactionData::Genesis);
        assert!(!dag.walk_search(&transaction, prev_milestone.get_hash(), 0, &mut |_| {}, &mut |_| {}));
    }

    #[test]
    fn test_add_milestone() {
        let mut dag = BlockDAG::default();
        let mut key = PrivateKey::new(&SHA512_256);
        let data = TransactionData::GenContract(ContractSource::new(vec![], 0));
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH,
            vec![], 0, BASE_NONCE, data);
        transaction.sign(&mut key);
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
        match dag.verify_milestone(transaction) {
            Ok(chain) => {
                assert_eq!(1, chain.len());
                assert_eq!(chain[0], dag.get_transaction(TRUNK_HASH).unwrap());
            },
            Err(err) => panic!("Unexpected missing transactions: {:?}", err)
        }
    }

    #[test]
    fn test_get_confirmation_status() {
        let mut dag = BlockDAG::default();
        assert_eq!(dag.get_confirmation_status(TRUNK_HASH), TransactionStatus::Accepted);
        assert_eq!(dag.get_confirmation_status(BRANCH_HASH), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(10), TransactionStatus::Rejected);

        let mut key = PrivateKey::new(&SHA512_256);
        let data = TransactionData::GenContract(ContractSource::new(vec![], 0));
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH,
            vec![], 0, BASE_NONCE, data);
        transaction.sign(&mut key);
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(transaction.get_hash()), TransactionStatus::Pending);
    }
}
