use std::collections::HashMap;

use rand::{thread_rng, Rng};

use dag::contract::{state::ContractStateStorage, Contract, ContractValue};
use dag::milestone::pending::{MilestoneSignature, MilestoneTracker};
use dag::milestone::Milestone;
use dag::storage::map::{Map, OOB};
use dag::storage::mpt::{node::Node, MerklePatriciaTree};
use dag::transaction::{
    data::TransactionData, error::TransactionError, updates::TransactionUpdates, Transaction,
};

use super::incomplete_chain::IncompleteChain;

use security::hash::proof::valid_proof;

use util::types::{TransactionHashes, TransactionStatus};

const GENESIS_HASH: u64 = 0;

const MILESTONE_NONCE_MIN: u32 = 100_000;
const MILESTONE_NONCE_MAX: u32 = 200_000;

pub trait TransactionStorage = Map<u64, Transaction>;
pub trait ContractStorage = Map<u64, Contract>;

pub struct BlockDAG<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage> {
    transactions: T,
    pending_transactions: HashMap<u64, Transaction>,
    contracts: C,
    storage: MerklePatriciaTree<ContractValue, M>,
    milestones: MilestoneTracker,
    tips: Vec<u64>,
}

impl<
        M: ContractStateStorage + Default,
        T: TransactionStorage + Default,
        C: ContractStorage + Default,
    > Default for BlockDAG<M, T, C>
{
    fn default() -> Self {
        Self::new(T::default(), C::default(), M::default())
    }
}

impl<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage> BlockDAG<M, T, C> {
    #[allow(unused_must_use)]
    pub fn new(transaction_storage: T, contract_storage: C, state_storage: M) -> Self {
        let storage = MerklePatriciaTree::new(state_storage);
        let default_root = storage.default_root();

        let genesis_transaction = Transaction::new(
            GENESIS_HASH,
            GENESIS_HASH,
            vec![],
            0,
            0,
            0,
            default_root,
            TransactionData::Genesis,
        );
        let genesis_milestone = Milestone::new(GENESIS_HASH, genesis_transaction.clone());

        let mut dag = BlockDAG {
            transactions: transaction_storage,
            pending_transactions: HashMap::default(),
            contracts: contract_storage,
            storage,
            milestones: MilestoneTracker::new(genesis_milestone),
            tips: Vec::new(),
        };

        let genesis_transaction_hash = genesis_transaction.get_hash();
        let genesis_branch = Transaction::new(
            genesis_transaction_hash,
            genesis_transaction_hash,
            vec![],
            0,
            0,
            0,
            default_root,
            TransactionData::Genesis,
        );
        let genesis_branch_hash = genesis_branch.get_hash();

        dag.transactions
            .set(genesis_transaction_hash, genesis_transaction);
        dag.pending_transactions
            .set(genesis_branch_hash, genesis_branch);
        dag.tips.push(genesis_transaction_hash);
        dag.tips.push(genesis_branch_hash);

        dag
    }

    /// Try to add a transaction to the dag
    ///
    /// Calling this function checks the validity of the transaction against
    /// the local transactions and contracts
    ///
    /// If the transaction is not valid, either because the proof of work
    /// is invalid, or because one of the referenced transactions does not
    /// exist, the function will return TransactionStatus::Rejected
    pub fn try_add_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionUpdates, TransactionError> {
        let branch_transaction;
        let trunk_transaction;
        if let Some(trunk_handle) = self.get_transaction(transaction.get_trunk_hash()) {
            if let Some(branch_handle) = self.get_transaction(transaction.get_branch_hash()) {
                let trunk = trunk_handle.borrow();
                let branch = branch_handle.borrow();
                if !valid_proof(
                    trunk.get_nonce(),
                    branch.get_nonce(),
                    transaction.get_nonce(),
                ) {
                    return Err(TransactionError::Rejected("Invalid nonce".into()));
                }
                trunk_transaction = trunk.clone();
                branch_transaction = branch.clone();
            } else {
                return Err(TransactionError::Rejected(
                    "Branch transaction not found".into(),
                ));
            }
        } else {
            return Err(TransactionError::Rejected(
                "Trunk transaction not found".into(),
            ));
        }

        // Verify the transaction's signature
        if !transaction.verify() {
            return Err(TransactionError::Rejected("Invalid signature".into()));
        }

        let ref_hashes = transaction.get_ref_hashes();
        let mut referenced = Vec::with_capacity(ref_hashes.len() + 2);
        referenced.push(trunk_transaction.get_hash());
        referenced.push(branch_transaction.get_hash());
        for hash in ref_hashes {
            if let Some(t) = self.get_transaction(hash) {
                referenced.push(t.get_hash());
            } else {
                return Err(TransactionError::Rejected(
                    "Referenced transaction not found".into(),
                ));
            }
        }

        let hash = transaction.get_hash();

        let mut updates = TransactionUpdates::new(referenced);

        // Process the transaction's data
        match transaction.get_data() {
            TransactionData::Genesis => {
                return Err(TransactionError::Rejected("Genesis transaction".into()))
            }
            TransactionData::GenContract(src) => {
                if transaction.get_contract() != 0 {
                    return Err(TransactionError::Rejected("Invalid gen contract id".into()));
                }
                // Generate a new contract
                match Contract::new(src.clone(), hash, &self.storage, transaction.get_root()) {
                    Ok((contract, node_updates)) => {
                        updates.add_contract(contract);
                        updates.add_node_updates(node_updates);
                    }
                    Err(_) => return Err(TransactionError::Rejected("Invalid contract".into())),
                }
            }
            TransactionData::ExecContract(func_name, args) => {
                if transaction.get_contract() != trunk_transaction.get_contract()
                    && trunk_transaction.get_contract() != 0
                {
                    return Err(TransactionError::Rejected("Invalid contract id".into()));
                }
                if let Ok(contract) = self.contracts.get(&transaction.get_contract()) {
                    match contract.exec(func_name, args, &self.storage, transaction.get_root()) {
                        Ok((_val, node_updates)) => {
                            updates.add_node_updates(node_updates);
                        }
                        Err(err) => {
                            return Err(TransactionError::Rejected(format!(
                                "Function failed to execute: {:?}",
                                err
                            )));
                        }
                    }
                } else {
                    return Err(TransactionError::Rejected("Contract not found".into()));
                }
            }
            TransactionData::Empty => {}
        };

        Ok(updates)
    }

    /// inserts the new transaction into the list
    /// of active tips, and moves all transactions it references from
    /// list of active tips to the list of transactions.
    pub fn commit_transaction(
        &mut self,
        transaction: Transaction,
        updates: TransactionUpdates,
    ) -> Result<TransactionStatus, TransactionError> {
        let hash = transaction.get_hash();

        if let Some(updates) = updates.node_updates {
            self.storage.commit_set(updates)?;
        }
        if let Some(contract) = updates.contract {
            self.contracts.set(hash, contract)?;
        }
        for t in updates.referenced {
            self.tips.remove_item(&t);
        }

        let mut res = TransactionStatus::Pending;

        if transaction.get_nonce() > MILESTONE_NONCE_MIN
            && transaction.get_nonce() < MILESTONE_NONCE_MAX
            && self.milestones.new_milestone(transaction.clone())
        {
            res = TransactionStatus::Milestone;
        }

        self.pending_transactions.set(hash, transaction)?;
        self.tips.push(hash);

        return Ok(res);
    }

    /// Add a confirmed milestone to the list of milestones
    ///
    /// Walks backward on the graph searching for the previous milestone
    ///
    /// If the milestone is found, return the chain of transactions to it
    ///
    /// If the milestone is not found, return and IncompleteChain error with any
    /// transactions that were not found locally
    pub fn verify_milestone(
        &self,
        transaction: Transaction,
    ) -> Result<Vec<Transaction>, IncompleteChain> {
        let prev_milestone = self.milestones.get_head_milestone();
        let mut transaction_chain: Vec<Transaction> = Vec::new();
        let mut missing_hashes: Vec<u64> = Vec::new();

        let transaction_found = {
            let chain_function = &mut |transaction: &Transaction| {
                transaction_chain.push(transaction.clone());
            };
            let not_found_function = &mut |hash: u64| {
                missing_hashes.push(hash);
            };

            self.walk_search(
                &transaction,
                prev_milestone.get_hash(),
                prev_milestone.get_timestamp(),
                chain_function,
                not_found_function,
            )
        };

        if transaction_found {
            Ok(transaction_chain)
        } else {
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
    fn walk_search<F, G>(
        &self,
        transaction: &Transaction,
        hash: u64,
        timestamp: u64,
        chain_function: &mut F,
        not_found_function: &mut G,
    ) -> bool
    where
        F: FnMut(&Transaction),
        G: FnMut(u64),
    {
        if transaction.get_timestamp() < timestamp {
            return false;
        }
        for transaction_hash in transaction.get_all_refs() {
            if let Some(transaction_handle) = self.get_transaction(transaction_hash) {
                let transaction = transaction_handle.borrow();
                if transaction_hash == hash {
                    // This is the transaction we are looking for, return
                    return true;
                }
                if self.walk_search(
                    &transaction,
                    hash,
                    timestamp,
                    chain_function,
                    not_found_function,
                ) {
                    // Found the transaction somewhere along this chain
                    chain_function(&transaction);
                    return true;
                }
            } else {
                not_found_function(transaction_hash);
            }
        }
        false
    }

    /// Move all transactions referenced by transaction from
    /// pending_transactions to transactions
    #[allow(unused_must_use)]
    fn confirm_transactions(&mut self, transaction: &Transaction) {
        for transaction_hash in transaction.get_all_refs() {
            if let Some(pending_transaction) = self.pending_transactions.remove(&transaction_hash) {
                self.confirm_transactions(&pending_transaction);
                self.transactions.set(transaction_hash, pending_transaction);
            }
        }
    }

    /// Returns the transaction specified by hash
    pub fn get_transaction<'a>(&'a self, hash: u64) -> Option<OOB<'a, Transaction>> {
        self.pending_transactions
            .get(&hash)
            .map_or(self.transactions.get(&hash).ok(), |pending_transaction| {
                Some(OOB::Borrowed(pending_transaction))
            })
    }

    /// Get the confirmation status of a transaction specified by hash
    pub fn get_confirmation_status(&self, hash: u64) -> TransactionStatus {
        if self.pending_transactions.get(&hash).is_some() {
            return TransactionStatus::Pending;
        }
        if self.transactions.get(&hash).is_ok() {
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
        let (trunk_tip, branch_tip) = if self.tips.len() > 1 {
            // Randomly select two unique transactions from the tips
            let mut rng = thread_rng();
            let trunk_tip_idx = rng.gen_range(0, self.tips.len());
            let mut branch_tip_idx = rng.gen_range(0, self.tips.len());
            while branch_tip_idx == trunk_tip_idx {
                branch_tip_idx = rng.gen_range(0, self.tips.len());
            }

            (self.tips[trunk_tip_idx], self.tips[branch_tip_idx])
        } else {
            let trunk_tip = self.tips[0];
            (
                trunk_tip,
                self.get_transaction(trunk_tip).unwrap().get_branch_hash(),
            )
        };

        TransactionHashes::new(trunk_tip, branch_tip)
    }

    pub fn get_contract<'a>(&'a self, id: u64) -> Option<OOB<Contract>> {
        self.contracts.get(&id).ok()
    }

    pub fn get_mpt_node<'a>(&'a self, id: u64) -> Option<OOB<Node<ContractValue>>> {
        self.storage.nodes.get(&id).ok()
    }

    pub fn get_mpt_default_root(&self) -> u64 {
        self.storage.default_root()
    }
}

impl<M: ContractStateStorage, T: TransactionStorage> BlockDAG<M, T, HashMap<u64, Contract>> {
    // Get the hash id's of all the contracts stored on the dag
    pub fn get_contracts(&self) -> Vec<u64> {
        self.contracts.keys().cloned().collect()
    }
}

#[cfg(test)]
impl<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage> BlockDAG<M, T, C> {
    fn force_add_transaction(&mut self, transaction: Transaction) {
        let hash = transaction.get_hash();
        self.pending_transactions.insert(hash, transaction);
        self.tips.push(hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    use dag::contract::{source::ContractSource, ContractValue};
    use dag::transaction::Transaction;

    use security::hash::proof::proof_of_work;
    use security::keys::PrivateKey;
    use security::ring::digest::SHA512_256;

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: u64 = 7994361212180723510;
    const BRANCH_HASH: u64 = 5285319433948766311;

    const BASE_NONCE: u32 = 132;

    fn insert_transaction<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage>(
        dag: &mut BlockDAG<M, T, C>,
        branch: u64,
        trunk: u64,
        contract: u64,
        data: TransactionData,
    ) -> Transaction {
        let transaction = Transaction::new(branch, trunk, Vec::new(), contract, 0, 0, 0, data);
        dag.force_add_transaction(transaction.clone());
        transaction
    }

    #[test]
    fn test_genesis_transactions() {
        let dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let tips = dag.get_tips();

        if tips.trunk_hash == TRUNK_HASH {
            assert_eq!(tips.branch_hash, BRANCH_HASH);
        } else {
            assert_eq!(tips.trunk_hash, BRANCH_HASH);
            assert_eq!(tips.branch_hash, TRUNK_HASH);
        }
    }

    #[test]
    fn test_add_transaction() {
        let mut dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let mut key = PrivateKey::new(&SHA512_256);
        let data = TransactionData::Empty;
        let mut transaction =
            Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], 0, BASE_NONCE, 0, data);
        transaction.sign(&mut key);
        assert!(transaction.verify());
        let updates = dag.try_add_transaction(&transaction).unwrap();
        assert_eq!(
            Ok(TransactionStatus::Pending),
            dag.commit_transaction(transaction.clone(), updates)
        );

        let tips = dag.get_tips();
        assert_eq!(tips.trunk_hash, transaction.get_hash());
        assert_eq!(tips.branch_hash, transaction.get_branch_hash());
        drop(tips);

        let bad_transaction =
            Transaction::create(10, BRANCH_HASH, vec![], 0, 0, 0, TransactionData::Genesis);
        assert_eq!(
            dag.try_add_transaction(&bad_transaction),
            Err(TransactionError::Rejected(
                "Branch transaction not found".into()
            ))
        );
    }

    #[test]
    fn test_walk_search() {
        let dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let prev_milestone = dag.milestones.get_head_milestone();

        let transaction = Transaction::create(
            TRUNK_HASH,
            BRANCH_HASH,
            vec![],
            0,
            0,
            0,
            TransactionData::Genesis,
        );
        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction =
            Transaction::create(TRUNK_HASH, 0, vec![], 0, 0, 0, TransactionData::Genesis);
        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction =
            Transaction::create(0, BRANCH_HASH, vec![], 0, 0, 0, TransactionData::Genesis);
        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction = Transaction::create(0, 0, vec![], 0, 0, 0, TransactionData::Genesis);
        assert!(!dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));
    }

    #[test]
    fn test_add_milestone() {
        let mut dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let data = TransactionData::GenContract(ContractSource::new(&vec![]));
        let middle_transaction = insert_transaction(&mut dag, 0, TRUNK_HASH, 1, data.clone());
        let transaction = insert_transaction(&mut dag, 0, middle_transaction.get_hash(), 1, data);

        match dag.verify_milestone(transaction) {
            Ok(chain) => {
                assert_eq!(1, chain.len());
                assert_eq!(chain[0].get_hash(), middle_transaction.get_hash());
            }
            Err(err) => panic!("Unexpected missing transactions: {:?}", err),
        }
    }

    #[test]
    fn test_get_confirmation_status() {
        let mut dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        assert_eq!(
            dag.get_confirmation_status(TRUNK_HASH),
            TransactionStatus::Accepted
        );
        assert_eq!(
            dag.get_confirmation_status(BRANCH_HASH),
            TransactionStatus::Pending
        );
        assert_eq!(
            dag.get_confirmation_status(10),
            TransactionStatus::Rejected("Not accepted".into())
        );

        let mut key = PrivateKey::new(&SHA512_256);
        let data = TransactionData::Empty;
        let mut transaction =
            Transaction::create(TRUNK_HASH, BRANCH_HASH, vec![], 0, BASE_NONCE, 0, data);
        transaction.sign(&mut key);
        let updates = dag.try_add_transaction(&transaction).unwrap();
        assert_eq!(
            Ok(TransactionStatus::Pending),
            dag.commit_transaction(transaction.clone(), updates)
        );
        assert_eq!(
            dag.get_confirmation_status(transaction.get_hash()),
            TransactionStatus::Pending
        );
    }

    use dag::contract::state::get_key;

    #[test]
    fn test_gen_exec_contract_transaction() {
        // Load example contract file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/contracts/api_test.wasm");
        let filename = d.to_str().unwrap().to_string();
        let mut file = File::open(filename).expect("Could not open test file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf)
            .expect("Could not read test file");

        let mut dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let mpt_root = dag.get_transaction(TRUNK_HASH).unwrap().get_root();
        let contract_id;
        let trunk_hash;
        let branch_hash;
        {
            let mut key = PrivateKey::new(&SHA512_256);
            let data = TransactionData::GenContract(ContractSource::new(&buf));
            let mut transaction = Transaction::create(
                TRUNK_HASH,
                BRANCH_HASH,
                vec![],
                0,
                BASE_NONCE,
                mpt_root,
                data,
            );
            transaction.sign(&mut key);
            assert!(transaction.verify());
            let updates = dag.try_add_transaction(&transaction).unwrap();
            assert_eq!(
                dag.commit_transaction(transaction.clone(), updates)
                    .unwrap(),
                TransactionStatus::Pending
            );
            contract_id = transaction.get_hash();

            trunk_hash = transaction.get_hash();
            branch_hash = transaction.get_branch_hash();
        }
        {
            let new_value = 2;
            let branch_nonce = dag.get_transaction(branch_hash).unwrap().get_nonce();
            let trunk_nonce = dag.get_transaction(trunk_hash).unwrap().get_nonce();
            let nonce = proof_of_work(trunk_nonce, branch_nonce);
            let mut key = PrivateKey::new(&SHA512_256);
            let data = TransactionData::ExecContract(
                "set_u32".into(),
                vec![ContractValue::U32(0), ContractValue::U32(new_value)],
            );

            let mut transaction = Transaction::create(
                branch_hash,
                trunk_hash,
                vec![],
                contract_id,
                nonce,
                mpt_root,
                data,
            );
            transaction.sign(&mut key);
            assert!(transaction.verify());

            let updates = dag.try_add_transaction(&transaction).unwrap();

            let new_root = updates.get_storage_root().unwrap();
            assert_eq!(
                dag.commit_transaction(transaction.clone(), updates)
                    .unwrap(),
                TransactionStatus::Pending
            );

            assert_eq!(
                Ok(OOB::Borrowed(&ContractValue::U32(new_value))),
                dag.storage.get(new_root, get_key(0, contract_id))
            );
        }
    }
}
