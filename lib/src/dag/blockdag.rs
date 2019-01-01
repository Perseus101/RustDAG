use std::collections::{HashSet, HashMap};

use rand::{Rng,thread_rng};

use dag::transaction::{Transaction, data::TransactionData};
use dag::contract::{Contract, state::persistent::PersistentCachedContractState as PersistentState};
use dag::milestone::Milestone;
use dag::milestone::pending::{
    MilestoneSignature,
    MilestoneTracker
};

use super::incomplete_chain::IncompleteChain;

use security::hash::proof::valid_proof;

use util::types::{TransactionHashes,TransactionStatus};

const GENESIS_HASH: u64 = 0;

const MILESTONE_NONCE_MIN: u32 = 100000;
const MILESTONE_NONCE_MAX: u32 = 200000;

pub enum PendingTransaction {
    Transaction(Transaction),
    State(Transaction, PersistentState)
}

impl From<Transaction> for PendingTransaction {
    fn from(transaction: Transaction) -> Self {
        PendingTransaction::Transaction(transaction)
    }
}

pub struct BlockDAG {
    transactions: HashMap<u64, Transaction>,
    pending_transactions: HashMap<u64, PendingTransaction>,
    contracts: HashMap<u64, Contract>,
    milestones: MilestoneTracker,
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
            milestones: MilestoneTracker::new(genesis_milestone),
            tips: Vec::new(),
		};

        let genesis_transaction_hash = genesis_transaction.get_hash();
        let genesis_branch = Transaction::new(genesis_transaction_hash,
            genesis_transaction_hash, vec![], 0, 0, 0, TransactionData::Genesis);
        let genesis_branch_hash = genesis_branch.get_hash();

        dag.transactions.insert(genesis_transaction_hash, genesis_transaction);
        dag.pending_transactions.insert(genesis_branch_hash, genesis_branch.into());
        dag.tips.push(genesis_transaction_hash);
        dag.tips.push(genesis_branch_hash);

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
        let branch_transaction;
        let trunk_transaction;
        if let Some(trunk) = self.get_transaction(transaction.get_trunk_hash()) {
            if let Some(branch) = self.get_transaction(transaction.get_branch_hash()) {
                if !valid_proof(trunk.get_nonce(), branch.get_nonce(), transaction.get_nonce()) {
                    return TransactionStatus::Rejected("Invalid nonce".into());
                }
                trunk_transaction = trunk.clone();
                branch_transaction = branch.clone();
            }
            else { return TransactionStatus::Rejected("Branch transaction not found".into()); }
        }
        else { return TransactionStatus::Rejected("Trunk transaction not found".into()); }

        // Verify the transaction's signature
        if !transaction.verify() { return TransactionStatus::Rejected("Invalid signature".into()); }

        let hash = transaction.get_hash();

        // Process the transaction's data
        let pending_transaction = match transaction.get_data() {
            TransactionData::Genesis => return TransactionStatus::Rejected("Genesis transaction".into()),
            TransactionData::GenContract(src) => {
                if transaction.get_contract() != 0 {
                    return TransactionStatus::Rejected("Invalid gen contract id".into());
                }
                // Generate a new contract
                match Contract::new(src.clone()) {
                    Ok(contract) => { self.contracts.insert(hash, contract); },
                    Err(_) => return TransactionStatus::Rejected("Invalid contract".into()),
                }
                PendingTransaction::Transaction(transaction.clone())
            },
            TransactionData::ExecContract(func_name, args) => {
                if transaction.get_contract() != trunk_transaction.get_contract()
                        && trunk_transaction.get_contract() != 0 {
                    return TransactionStatus::Rejected("Invalid contract id".into());
                }

                // Attempt to load the contract state and execute the requested function
                match self.contracts.get(&transaction.get_contract()) {
                    None => return TransactionStatus::Rejected("Contract not found".into()),
                    Some(contract) => {
                        match self.pending_transactions.get(&transaction.get_trunk_hash()) {
                            Some(PendingTransaction::Transaction(_)) =>
                                return TransactionStatus::Rejected("No pending state".into()),
                            Some(PendingTransaction::State(_, state)) => {
                                // Found transaction with cached state
                                match contract.exec_persisted(&func_name, &args, state.clone()) {
                                    Err(_) => return TransactionStatus::Rejected("Failed to execute persisted contract".into()),
                                    Ok((_, persistent)) => {
                                        PendingTransaction::State(transaction.clone(), persistent)
                                    }
                                }
                            },
                            None => {
                                // No transaction with cached state, load directly from the contract
                                match contract.exec(&func_name, &args) {
                                    Err(_) => return TransactionStatus::Rejected("Failed to execute contract".into()),
                                    Ok((_, persistent)) => {
                                        PendingTransaction::State(transaction.clone(), persistent)
                                    }
                                }
                            }
                        }
                    },
                }
            },
            TransactionData::Empty => {
                PendingTransaction::Transaction(transaction.clone())
            }
        };

        let ref_hashes = transaction.get_ref_hashes();
        let mut referenced = Vec::with_capacity(ref_hashes.len() + 2);
        referenced.push(trunk_transaction.get_hash());
        referenced.push(branch_transaction.get_hash());
        for hash in ref_hashes {
            if let Some(t) = self.get_transaction(hash) {
                referenced.push(t.get_hash());
            }
            else { return TransactionStatus::Rejected("Referenced transaction not found".into()); }
        }

        for t in referenced {
            self.tips.remove_item(&t);
        }
        self.pending_transactions.insert(hash, pending_transaction);
        self.tips.push(hash);

        if transaction.get_nonce() > MILESTONE_NONCE_MIN &&
            transaction.get_nonce() < MILESTONE_NONCE_MAX {
            if self.milestones.new_milestone(transaction.clone()) {
                return TransactionStatus::Milestone;
            }
        }
        TransactionStatus::Pending
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
        let prev_milestone = self.milestones.get_head_milestone();
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
    pub fn process_chain(&mut self, milestone: u64, chain: Vec<Transaction>) -> bool {
        for transaction in chain.into_iter() {
            if let Err(_err) = self.milestones.new_chain(milestone, transaction) {
                // TODO Log error
                return false;
            }
        }
        true
    }

    /// Add a signature to the current pending milestone
    pub fn add_pending_signature(&mut self, signature: MilestoneSignature) -> bool {
        match self.milestones.sign(signature) {
            Ok(Some(milestone)) => {
                self.confirm_transactions(milestone.get_transaction());
                true
            }
            Ok(None) => true,
            Err(_err) => {
                // TODO Log error
                false
            }
        }
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
        let mut states = HashSet::new();
        self.confirm_transactions_helper(transaction, &mut states)
    }

    /// Helper function for confirming transactions
    fn confirm_transactions_helper(&mut self, transaction: &Transaction, states: &mut HashSet<u64>) {
        for transaction_hash in transaction.get_all_refs() {
            if let Some(pending_transaction) = self.pending_transactions.remove(&transaction_hash) {
                let transaction = match pending_transaction {
                    PendingTransaction::Transaction(t) => t,
                    PendingTransaction::State(t, state) => {
                        let contract_id = t.get_contract();
                        if !states.contains(&contract_id) {
                            if let Some(mut contract) = self.contracts.get_mut(&contract_id) {
                                states.insert(contract_id);
                                // XXX handle error rather than panic
                                contract.writeback(state).expect("Failed to write out contract state");
                            }
                            else if contract_id == 0 {
                                states.insert(contract_id);
                            }
                            else {
                                // XXX This is really bad.
                                panic!("Attempted to confirm a transaction with an invalid contract");
                            }
                        }
                        t
                    }
                };
                self.confirm_transactions_helper(&transaction, states);
                self.transactions.insert(transaction_hash, transaction);
            }
        }
    }

    /// Returns the transaction specified by hash
    pub fn get_transaction(&self, hash: u64) -> Option<&Transaction> {
        self.pending_transactions.get(&hash).map_or(
            self.transactions.get(&hash), |pending_transaction| {
                Some(match pending_transaction {
                    PendingTransaction::Transaction(t) => t,
                    PendingTransaction::State(t, _) => t,
                })
            }
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
        TransactionStatus::Rejected("Not accepted".into())
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

    pub fn get_contract(&self, id: u64) -> Option<&Contract> {
        self.contracts.get(&id)
    }

    // Get the hash id's of all the contracts stored on the dag
    pub fn get_contracts(&self) -> Vec<u64> {
        self.contracts.keys().map(|x| *x).collect()
    }
}

#[cfg(test)]
impl BlockDAG {
    fn force_add_transaction(&mut self, transaction: Transaction) {
        let hash = transaction.get_hash();
        self.pending_transactions.insert(hash, PendingTransaction::Transaction(transaction));
        self.tips.push(hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;

    use dag::transaction::Transaction;
    use dag::contract::{ContractValue, source::ContractSource};

    use security::keys::PrivateKey;
    use security::ring::digest::SHA512_256;
    use security::hash::proof::proof_of_work;

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: u64 = 18437817136469695293;
    const BRANCH_HASH: u64 = 6043537212972274484;

    const BASE_NONCE: u32 = 18722;

    fn insert_transaction(dag: &mut BlockDAG, branch: u64, trunk: u64,
            contract: u64, data: TransactionData) -> Transaction {
        let transaction = Transaction::new(branch, trunk, Vec::new(),
            contract, 0, 0, data);
        dag.force_add_transaction(transaction.clone());
        transaction
    }

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
        let data = TransactionData::Empty;
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
        assert_eq!(dag.add_transaction(&bad_transaction), TransactionStatus::Rejected("Branch transaction not found".into()));
    }

    #[test]
    fn test_walk_search() {
        let dag = BlockDAG::default();
        let prev_milestone = dag.milestones.get_head_milestone();

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
        let data = TransactionData::GenContract(ContractSource::new(&vec![]));
        let middle_transaction = insert_transaction(&mut dag, 0, TRUNK_HASH, 1, data.clone());
        let transaction = insert_transaction(&mut dag, 0,
            middle_transaction.get_hash(), 1, data);

        match dag.verify_milestone(transaction) {
            Ok(chain) => {
                assert_eq!(1, chain.len());
                assert_eq!(chain[0].get_hash(), middle_transaction.get_hash());
            },
            Err(err) => panic!("Unexpected missing transactions: {:?}", err)
        }
    }

    #[test]
    fn test_get_confirmation_status() {
        let mut dag = BlockDAG::default();
        assert_eq!(dag.get_confirmation_status(TRUNK_HASH), TransactionStatus::Accepted);
        assert_eq!(dag.get_confirmation_status(BRANCH_HASH), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(10), TransactionStatus::Rejected("Not accepted".into()));

        let mut key = PrivateKey::new(&SHA512_256);
        let data = TransactionData::Empty;
        let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH,
            vec![], 0, BASE_NONCE, data);
        transaction.sign(&mut key);
        assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
        assert_eq!(dag.get_confirmation_status(transaction.get_hash()), TransactionStatus::Pending);
    }

    #[test]
    #[ignore]
    fn test_gen_exec_contract_transaction() {
        // Load example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/full_api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).expect("Could not read test file");

        let mut dag = BlockDAG::default();
        let contract_id;
        let trunk_hash;
        let branch_hash;
        {
            let mut key = PrivateKey::new(&SHA512_256);
            let data = TransactionData::GenContract(ContractSource::new(&buf));
            let mut transaction = Transaction::create(TRUNK_HASH, BRANCH_HASH,
                vec![], 0, BASE_NONCE, data);
            transaction.sign(&mut key);
            assert!(transaction.verify());
            assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);
            contract_id = transaction.get_hash();

            // Contracts can't be run until the initial transaction is confirmed
            // Create a new fake milestone transaction to confirm transaction
            let branch_nonce = dag.get_transaction(BRANCH_HASH).unwrap().get_nonce();
            let nonce = proof_of_work(BASE_NONCE, branch_nonce);
            let mut fake_milestone = Transaction::create(BRANCH_HASH,
                transaction.get_hash(), vec![], 0, nonce, TransactionData::Empty);
            let mut key = PrivateKey::new(&SHA512_256);
            fake_milestone.sign(&mut key);
            assert_eq!(dag.add_transaction(&fake_milestone), TransactionStatus::Pending);
            dag.confirm_transactions(&fake_milestone);
            assert_eq!(TransactionStatus::Accepted,
                dag.get_confirmation_status(transaction.get_hash()));

            trunk_hash = transaction.get_hash();
            branch_hash = fake_milestone.get_hash();
        }
        {
            let new_value = 2;
            let branch_nonce = dag.get_transaction(branch_hash).unwrap().get_nonce();
            let trunk_nonce = dag.get_transaction(trunk_hash).unwrap().get_nonce();
            let nonce = proof_of_work(trunk_nonce, branch_nonce);
            let mut key = PrivateKey::new(&SHA512_256);
            let data = TransactionData::ExecContract("set_u32".to_owned(),
                vec![ContractValue::U32(0), ContractValue::U32(new_value)]);

            let mut transaction = Transaction::create(branch_hash, trunk_hash,
                vec![], contract_id, nonce, data);
            transaction.sign(&mut key);

            assert_eq!(dag.add_transaction(&transaction), TransactionStatus::Pending);

            // Create another fake milestone to confirm
            let nonce = proof_of_work(branch_nonce, transaction.get_nonce());
            let mut fake_milestone = Transaction::create(transaction.get_hash(),
                branch_hash, vec![], 0, nonce, TransactionData::Empty);
            let mut key = PrivateKey::new(&SHA512_256);
            fake_milestone.sign(&mut key);
            assert_eq!(dag.add_transaction(&fake_milestone), TransactionStatus::Pending);
            dag.confirm_transactions(&fake_milestone);
            assert_eq!(TransactionStatus::Accepted,
                dag.get_confirmation_status(transaction.get_hash()));

            let contract = dag.get_contract(contract_id).unwrap();
            assert_eq!(new_value, contract.get_state().int32[0]);
        }
    }
}
