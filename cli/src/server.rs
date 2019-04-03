use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::Read;

use rustdag_lib::{
    dag::{
        blockdag::BlockDAG,
        contract::{error::ContractError, source::ContractSource, ContractValue},
        error::BlockDAGError,
        transaction::{
            data::TransactionData, error::TransactionError, header::TransactionHeader, Transaction,
        },
    },
    security::{
        hash::proof::proof_of_work,
        keys::eddsa::{new_key_pair, EdDSAKeyPair},
    },
    util::{
        epoch_time,
        peer::{ContractPeer, MPTNodePeer, Peer, TransactionPeer},
        types::TransactionStatus,
    },
};

use crate::error::CliError;
use crate::merge_header::MergeHeader;
pub struct Server {
    peer: Peer,
    blockdag: BlockDAG<MPTNodePeer, TransactionPeer, ContractPeer>,
}

impl Server {
    pub fn new(server_url: &str) -> Self {
        let peer = Peer::new(String::from(server_url));
        let blockdag = peer.clone().into_remote_blockdag();
        Server { peer, blockdag }
    }

    fn find_merge_base(&self, t_a: Transaction, t_b: Transaction) -> Result<u64, CliError> {
        let mut set_a = HashSet::new();
        let mut set_b = HashSet::new();
        let mut queue_a = VecDeque::new();
        let mut queue_b = VecDeque::new();

        set_a.insert(t_a.get_hash());
        set_b.insert(t_b.get_hash());
        queue_a.push_back(t_a);
        queue_b.push_back(t_b);
        self._find_merge_base(&mut set_a, &mut set_b, &mut queue_a, &mut queue_b)
    }

    fn _find_merge_base(
        &self,
        set_a: &mut HashSet<u64>,
        set_b: &mut HashSet<u64>,
        queue_a: &mut VecDeque<Transaction>,
        queue_b: &mut VecDeque<Transaction>,
    ) -> Result<u64, CliError> {
        while let (Some(t_a), Some(t_b)) = (queue_a.pop_front(), queue_b.pop_front()) {
            if let Some(hash) = self._test_transaction(t_a, set_b, set_a, queue_a) {
                return Ok(hash);
            }
            if let Some(hash) = self._test_transaction(t_b, set_a, set_b, queue_b) {
                return Ok(hash);
            }
        }

        // In the unlikely event that one of the queues becomes empty
        // exhaust the other queue
        let (queue, test_set, add_set) = if queue_a.is_empty() {
            (queue_b, set_a, set_b)
        } else {
            (queue_a, set_b, set_a)
        };

        while let Some(transaction) = queue.pop_front() {
            if let Some(hash) = self._test_transaction(transaction, test_set, add_set, queue) {
                return Ok(hash);
            }
        }

        // Should be unreachable
        Err(BlockDAGError::MergeError.into())
    }

    fn _test_transaction(
        &self,
        transaction: Transaction,
        test_set: &mut HashSet<u64>,
        add_set: &mut HashSet<u64>,
        queue: &mut VecDeque<Transaction>,
    ) -> Option<u64> {
        for hash in transaction.get_all_refs().iter() {
            if test_set.contains(hash) {
                return Some(*hash);
            }
            add_set.insert(*hash);
            if let Ok(parent) = self.peer.get_transaction(*hash) {
                queue.push_back(parent);
            }
        }
        None
    }

    fn try_find_root(
        &mut self,
        trunk: &Transaction,
        branch: &Transaction,
    ) -> Result<MergeHeader, CliError> {
        // Search for the nearest common ancestor of branch and trunk
        let ancestor_hash = self.find_merge_base(trunk.clone(), branch.clone())?;
        let ancestor = self.peer.get_transaction(ancestor_hash)?;
        let ancestor_root = ancestor.get_merge_root();

        let trunk_updates = self.blockdag.try_add_transaction(&trunk)?;
        let trunk_root = trunk_updates
            .get_storage_root()
            .unwrap_or(trunk.get_merge_root());
        let branch_updates = self.blockdag.try_add_transaction(&branch)?;
        let branch_root = branch_updates
            .get_storage_root()
            .unwrap_or(branch.get_merge_root());

        if let Some(updates) = trunk_updates.get_node_updates() {
            self.blockdag.add_note_updates(updates)?;
        }
        if let Some(updates) = branch_updates.get_node_updates() {
            self.blockdag.add_note_updates(updates)?;
        }

        let updates = self
            .blockdag
            .try_merge(trunk_root, branch_root, ancestor_root)
            .ok_or_else(|| CliError::DagError(BlockDAGError::MergeError))?;
        let merge_root = updates.get_root_hash();
        self.blockdag.add_note_updates(updates)?;

        Ok(MergeHeader::new(
            trunk_root,
            branch_root,
            merge_root,
            ancestor_root,
        ))
    }

    fn create_transaction(
        &mut self,
        contract: u64,
        data: TransactionData,
        key: &EdDSAKeyPair,
    ) -> Result<Transaction, CliError> {
        // Search for valid tips
        let mut tip_hashes = self.peer.get_tips();
        let mut trunk = self.peer.get_transaction(tip_hashes.trunk_hash)?;
        let mut branch = self.peer.get_transaction(tip_hashes.branch_hash)?;
        let merge_header: MergeHeader;
        loop {
            match self.try_find_root(&trunk, &branch) {
                Ok(header) => {
                    merge_header = header;
                    break;
                }
                Err(_) => {}
            }
            tip_hashes = self.peer.get_tips();
            trunk = self.peer.get_transaction(tip_hashes.trunk_hash)?;
            branch = self.peer.get_transaction(tip_hashes.branch_hash)?;
        }

        // Create and sign transaction
        let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());
        let mut transaction = Transaction::new(
            merge_header.into_transaction_header(
                tip_hashes.branch_hash,
                tip_hashes.trunk_hash,
                contract,
                epoch_time(),
                nonce,
            ),
            data,
        );
        transaction.sign_eddsa(key);

        Ok(transaction)
    }

    pub fn empty_transaction(&mut self, use_default: bool) -> Result<(), CliError> {
        let key = new_key_pair().expect("Failed to create key pair");

        let transaction = if use_default {
            let tip_hashes = self.peer.get_tips();
            let trunk = self.peer.get_transaction(tip_hashes.trunk_hash)?;
            let branch = self.peer.get_transaction(tip_hashes.branch_hash)?;
            let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());
            let mut transaction = Transaction::new(
                TransactionHeader::new(
                    tip_hashes.branch_hash,
                    tip_hashes.trunk_hash,
                    0,
                    self.blockdag.get_mpt_default_root(),
                    self.blockdag.get_mpt_default_root(),
                    self.blockdag.get_mpt_default_root(),
                    self.blockdag.get_mpt_default_root(),
                    epoch_time(),
                    nonce,
                ),
                TransactionData::Empty,
            );
            transaction.sign_eddsa(&key);
            transaction
        } else {
            self.create_transaction(0, TransactionData::Empty, &key)?
        };

        if let Ok(TransactionStatus::Rejected(msg)) = self.peer.post_transaction(&transaction) {
            panic!("Transaction rejected: {}", msg);
        }
        Ok(())
    }

    pub fn run_contract(
        &mut self,
        contract: u64,
        function: String,
        args: Vec<String>,
    ) -> Result<Option<ContractValue>, CliError> {
        let key = new_key_pair().expect("Failed to create key pair");
        let data = TransactionData::ExecContract(
            function,
            args.iter()
                .map(|val| ContractValue::U64(val.parse::<u64>().expect("Invalid argument value")))
                .collect(),
        );
        let transaction = self.create_transaction(contract, data.clone(), &key)?;
        let val: Option<ContractValue>;
        if let TransactionData::ExecContract(function, args) = transaction.get_data() {
            let (value, updates) = match self.blockdag.execute_contract(
                contract,
                transaction.get_merge_root(),
                function,
                args,
            ) {
                Ok(data) => data,
                Err(TransactionError::Contract(ContractError::NoUpdates(value))) => {
                    return Ok(value);
                }
                Err(err) => return Err(BlockDAGError::from(err).into()),
            };
            if updates.get_root_hash() == transaction.get_merge_root() {
                println!("No changes");
                // TODO Send empty transaction
                return Ok(value);
            }
            val = value;
        } else {
            panic!("Data was not ExecContract")
        }

        if let Ok(TransactionStatus::Rejected(msg)) = self.peer.post_transaction(&transaction) {
            panic!("Transaction rejected: {}", msg);
        }
        Ok(val)
    }

    pub fn deploy_contract(&mut self, filename: &str) -> u64 {
        // Load contract
        let mut file = File::open(filename).expect("Could not open file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf)
            .expect("Could not read test file");
        let contract_src = ContractSource::new(&buf);

        let key = new_key_pair().expect("Failed to create key pair");
        let transaction = self
            .create_transaction(0, TransactionData::GenContract(contract_src), &key)
            .expect("Failed to create transaction");

        let contract_id = transaction.get_hash();

        if let Ok(TransactionStatus::Rejected(msg)) = self.peer.post_transaction(&transaction) {
            panic!("Contract rejected: {}", msg);
        }
        contract_id
    }
}
