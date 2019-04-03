use std::collections::HashMap;
use std::hash::BuildHasher;

use rand::{thread_rng, Rng};

use dag::contract::{state::ContractStateStorage, Contract, ContractValue};
use dag::error::BlockDAGError;
use dag::milestone::pending::{MilestoneSignature, MilestoneTracker};
use dag::milestone::Milestone;
use dag::storage::map::{Map, OOB};
use dag::storage::mpt::{node::Node, MerklePatriciaTree, NodeUpdates};
use dag::transaction::{
    data::TransactionData, error::TransactionError, header::TransactionHeader,
    updates::TransactionUpdates, Transaction,
};

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
            TransactionHeader::new(
                GENESIS_HASH,
                GENESIS_HASH,
                0,
                default_root,
                default_root,
                default_root,
                default_root,
                0,
                0,
            ),
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
            TransactionHeader::new(
                genesis_transaction_hash,
                genesis_transaction_hash,
                0,
                default_root,
                default_root,
                default_root,
                default_root,
                0,
                0,
            ),
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
    ) -> Result<TransactionUpdates, BlockDAGError> {
        let trunk = self
            .get_transaction(transaction.get_trunk_hash())
            .ok_or_else(|| TransactionError::Rejected("Trunk transaction not found".into()))?;
        let branch = self
            .get_transaction(transaction.get_branch_hash())
            .ok_or_else(|| TransactionError::Rejected("Branch transaction not found".into()))?;

        if !valid_proof(
            trunk.get_nonce(),
            branch.get_nonce(),
            transaction.get_nonce(),
        ) {
            return Err(TransactionError::Rejected("Invalid nonce".into()).into());
        }

        // Verify the transaction's signature
        if !transaction.verify() {
            return Err(TransactionError::Rejected("Invalid signature".into()).into());
        }

        let hash = transaction.get_hash();

        let mut updates = TransactionUpdates::new(transaction.get_all_refs());
        let merge_updates = self
            .try_merge(
                transaction.get_trunk_root(),
                transaction.get_branch_root(),
                transaction.get_ancestor_root(),
            )
            .ok_or_else(|| BlockDAGError::MergeError)?;
        let merge_root = merge_updates.get_root_hash();
        if merge_root != transaction.get_merge_root() {
            return Err(TransactionError::Rejected("Merge root does not match".into()).into());
        }

        // Process the transaction's data
        match transaction.get_data() {
            TransactionData::Genesis => {
                return Err(TransactionError::Rejected("Genesis transaction".into()).into());
            }
            TransactionData::GenContract(src) => {
                if transaction.get_contract() != 0 {
                    return Err(TransactionError::Rejected("Invalid gen contract id".into()).into());
                }
                // Generate a new contract
                match Contract::with_updates(src.clone(), hash, &self.storage, merge_updates) {
                    Ok((contract, node_updates)) => {
                        updates.add_contract(contract);
                        updates.add_node_updates(node_updates);
                    }
                    Err(_) => {
                        return Err(TransactionError::Rejected("Invalid contract".into()).into())
                    }
                }
            }
            TransactionData::ExecContract(func_name, args) => {
                let contract = transaction.get_contract();
                let (_, node_updates) = self.execute_contract_updates(
                    contract,
                    merge_root,
                    func_name,
                    args,
                    merge_updates,
                )?;
                updates.add_node_updates(node_updates);
            }
            TransactionData::Empty => {}
        };

        Ok(updates)
    }

    pub fn execute_contract(
        &self,
        contract: u64,
        root: u64,
        func_name: &str,
        args: &[ContractValue],
    ) -> Result<(Option<ContractValue>, NodeUpdates<ContractValue>), TransactionError> {
        let contract = self
            .contracts
            .get(&contract)
            .map_err(|_| TransactionError::Rejected("Contract not found".into()))?;
        contract
            .exec(func_name, args, &self.storage, root)
            .map_err(Into::into)
    }

    fn execute_contract_updates(
        &self,
        contract: u64,
        root: u64,
        func_name: &str,
        args: &[ContractValue],
        updates: NodeUpdates<ContractValue>,
    ) -> Result<(Option<ContractValue>, NodeUpdates<ContractValue>), TransactionError> {
        let contract = self
            .contracts
            .get(&contract)
            .map_err(|_| TransactionError::Rejected("Contract not found".into()))?;
        contract
            .exec_with_updates(func_name, args, &self.storage, root, updates)
            .map_err(Into::into)
    }

    /// inserts the new transaction into the list
    /// of active tips, and moves all transactions it references from
    /// list of active tips to the list of transactions.
    pub fn commit_transaction(
        &mut self,
        transaction: Transaction,
        updates: TransactionUpdates,
    ) -> Result<TransactionStatus, BlockDAGError> {
        let hash = transaction.get_hash();

        if let Some(updates) = updates.node_updates {
            self.storage.commit_set(updates)?;
        }
        if let Some(contract) = updates.contract {
            self.contracts.set(hash, contract)?;
        }

        let res = if transaction.get_nonce() > MILESTONE_NONCE_MIN
            && transaction.get_nonce() < MILESTONE_NONCE_MAX
            && self.milestones.new_milestone(transaction.clone())
        {
            TransactionStatus::Milestone
        } else {
            TransactionStatus::Pending
        };

        let refs = transaction.get_all_refs();
        self.pending_transactions.set(hash, transaction)?;

        for value in refs.iter() {
            self.tips.remove_item(value);
        }
        self.tips.push(hash);

        Ok(res)
    }

    pub fn try_merge(
        &self,
        root_a: u64,
        root_b: u64,
        root_ref: u64,
    ) -> Option<NodeUpdates<ContractValue>> {
        self.storage.try_merge(root_a, root_b, root_ref)
    }

    pub fn add_note_updates(
        &mut self,
        updates: NodeUpdates<ContractValue>,
    ) -> Result<(), BlockDAGError> {
        self.storage
            .commit_set(updates)
            .map_err(BlockDAGError::MapError)
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
    ) -> Result<Vec<Transaction>, BlockDAGError> {
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
            Err(BlockDAGError::IncompleteChain(missing_hashes))
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
        for t_hash in transaction.get_all_refs().iter() {
            let transaction_hash = *t_hash;
            if let Some(transaction_handle) = self.get_transaction(transaction_hash) {
                let transaction = transaction_handle.inner_ref();
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
        for transaction_hash in transaction.get_all_refs().iter() {
            if let Some(pending_transaction) = self.pending_transactions.remove(&transaction_hash) {
                self.confirm_transactions(&pending_transaction);
                self.transactions
                    .set(*transaction_hash, pending_transaction);
            }
        }
    }

    /// Returns the transaction specified by hash
    pub fn get_transaction(&self, hash: u64) -> Option<OOB<Transaction>> {
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

    pub fn get_contract(&self, id: u64) -> Option<OOB<Contract>> {
        self.contracts.get(&id).ok()
    }

    pub fn get_mpt_node(&self, id: u64) -> Option<OOB<Node<ContractValue>>> {
        self.storage.nodes.get(&id).ok()
    }

    pub fn get_mpt_default_root(&self) -> u64 {
        self.storage.default_root()
    }
}

impl<M: ContractStateStorage, T: TransactionStorage, S: BuildHasher> BlockDAG<M, T, HashMap<u64, Contract, S>> {
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
    use security::keys::eddsa::new_key_pair;

    // Hardcoded values for the hashes of the genesis transactions.
    // If the default genesis transactions change, these values must be updated.
    const TRUNK_HASH: u64 = 6971420668045886651;
    const BRANCH_HASH: u64 = 1752386085411038391;

    const BASE_NONCE: u32 = 132;

    fn insert_transaction<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage>(
        dag: &mut BlockDAG<M, T, C>,
        branch: u64,
        trunk: u64,
        contract: u64,
        data: TransactionData,
    ) -> Transaction {
        let transaction = Transaction::new(
            TransactionHeader::new(branch, trunk, contract, 0, 0, 0, 0, 0, 0),
            data,
        );
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
        let root = dag.get_mpt_default_root();
        let mut transaction = Transaction::new(
            TransactionHeader::new(
                BRANCH_HASH,
                TRUNK_HASH,
                0,
                root,
                root,
                root,
                root,
                0,
                BASE_NONCE,
            ),
            TransactionData::Empty,
        );
        let key = new_key_pair().unwrap();
        transaction.sign_eddsa(&key);
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

        let mut bad_transaction = Transaction::new(
            TransactionHeader::new(20, BRANCH_HASH, 0, 0, 0, 0, 0, 0, BASE_NONCE),
            TransactionData::Empty,
        );
        bad_transaction.sign_eddsa(&key);
        assert_eq!(
            dag.try_add_transaction(&bad_transaction),
            Err(TransactionError::Rejected("Branch transaction not found".into()).into())
        );
    }

    #[test]
    fn test_walk_search() {
        let dag = BlockDAG::<HashMap<_, _>, HashMap<_, _>, HashMap<_, _>>::default();
        let prev_milestone = dag.milestones.get_head_milestone();
        let transaction = Transaction::new(
            TransactionHeader::new(TRUNK_HASH, BRANCH_HASH, 0, 0, 0, 0, 0, 0, BASE_NONCE),
            TransactionData::Genesis,
        );

        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction = Transaction::new(
            TransactionHeader::new(TRUNK_HASH, 0, 0, 0, 0, 0, 0, 0, BASE_NONCE),
            TransactionData::Genesis,
        );

        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction = Transaction::new(
            TransactionHeader::new(0, BRANCH_HASH, 0, 0, 0, 0, 0, 0, BASE_NONCE),
            TransactionData::Genesis,
        );

        assert!(dag.walk_search(
            &transaction,
            prev_milestone.get_hash(),
            0,
            &mut |_| {},
            &mut |_| {}
        ));

        let transaction = Transaction::new(
            TransactionHeader::new(0, 0, 0, 0, 0, 0, 0, 0, BASE_NONCE),
            TransactionData::Genesis,
        );

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

        let root = dag.get_mpt_default_root();
        let mut transaction = Transaction::new(
            TransactionHeader::new(
                BRANCH_HASH,
                TRUNK_HASH,
                0,
                root,
                root,
                root,
                root,
                0,
                BASE_NONCE,
            ),
            TransactionData::Empty,
        );

        let key = new_key_pair().unwrap();
        transaction.sign_eddsa(&key);
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
        let mut root = dag.get_mpt_default_root();
        let key = new_key_pair().unwrap();
        let contract_id;
        let trunk_hash;
        let branch_hash;
        {
            let data = TransactionData::GenContract(ContractSource::new(&buf));
            let mut transaction = Transaction::new(
                TransactionHeader::new(
                    TRUNK_HASH,
                    BRANCH_HASH,
                    0,
                    root,
                    root,
                    root,
                    root,
                    0,
                    BASE_NONCE,
                ),
                data,
            );
            transaction.sign_eddsa(&key);
            assert!(transaction.verify());
            let updates = dag.try_add_transaction(&transaction).unwrap();
            root = updates.get_storage_root().unwrap();
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

            let data = TransactionData::ExecContract(
                "set_u32".into(),
                vec![ContractValue::U32(0), ContractValue::U32(new_value)],
            );

            let mut transaction = Transaction::new(
                TransactionHeader::new(
                    branch_hash,
                    trunk_hash,
                    contract_id,
                    root,
                    root,
                    root,
                    root,
                    0,
                    nonce,
                ),
                data,
            );
            transaction.sign_eddsa(&key);
            assert!(transaction.verify());

            let updates = dag
                .try_add_transaction(&transaction)
                .expect("Failed to execute contract");

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
