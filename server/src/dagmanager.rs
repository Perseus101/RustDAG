use std::collections::HashMap;
use std::marker::{Send, Sync};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use dag::{
    blockdag::{BlockDAG, ContractStorage, TransactionStorage},
    contract::{state::ContractStateStorage, Contract, ContractValue},
    milestone::pending::MilestoneSignature,
    storage::mpt::node::Node,
    transaction::{error::TransactionError, Transaction},
};
use peermanager::PeerManager;
use util::peer::Peer;
use util::types::{TransactionHashes, TransactionStatus};

pub type DAGManager = GenericDAGManager<
    HashMap<u64, Node<ContractValue>>,
    HashMap<u64, Transaction>,
    HashMap<u64, Contract>,
>;

pub struct GenericDAGManager<M: ContractStateStorage, T: TransactionStorage, C: ContractStorage> {
    dag: Arc<RwLock<BlockDAG<M, T, C>>>,
    peers: RwLock<PeerManager>,
}

impl<
        M: ContractStateStorage + Default,
        T: TransactionStorage + Default,
        C: ContractStorage + Default,
    > Default for GenericDAGManager<M, T, C>
{
    fn default() -> Self {
        GenericDAGManager {
            dag: Arc::new(RwLock::from(BlockDAG::default())),
            peers: RwLock::from(PeerManager::new()),
        }
    }
}

impl<
        M: 'static + ContractStateStorage + Send + Sync,
        T: 'static + TransactionStorage + Send + Sync,
    > GenericDAGManager<M, T, HashMap<u64, Contract>>
{
    pub fn get_tips(&self) -> TransactionHashes {
        self.dag.read().unwrap().get_tips()
    }

    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        self.dag
            .read()
            .unwrap()
            .get_transaction(hash)
            .and_then(|t| Some(t.clone()))
    }

    pub fn get_contract(&self, hash: u64) -> Option<Contract> {
        self.dag
            .read()
            .unwrap()
            .get_contract(hash)
            .and_then(|c| Some(c.clone()))
    }

    pub fn get_mpt_node(&self, hash: u64) -> Option<Node<ContractValue>> {
        self.dag
            .read()
            .unwrap()
            .get_mpt_node(hash)
            .and_then(|n| Some(n.clone()))
    }

    pub fn get_transaction_status(&self, hash: u64) -> TransactionStatus {
        self.dag.read().unwrap().get_confirmation_status(hash)
    }

    pub fn add_transaction(&self, transaction: Transaction) -> TransactionStatus {
        let hash = transaction.get_hash();
        {
            // Ignore any already known transactions
            let current_status = self.dag.read().unwrap().get_confirmation_status(hash);
            if current_status == TransactionStatus::Accepted
                || current_status == TransactionStatus::Pending
                || current_status == TransactionStatus::Milestone
            {
                return current_status;
            }
        }

        let dag_read = self.dag.read().unwrap();
        match dag_read.try_add_transaction(&transaction) {
            Ok(updates) => {
                drop(dag_read);
                let mut dag_write = self.dag.write().unwrap();
                match dag_write.commit_transaction(transaction.clone(), updates) {
                    Ok(status) => {
                        self.peers
                            .read()
                            .unwrap()
                            .map_peers(|peer| peer.post_transaction(&transaction));
                        if status == TransactionStatus::Milestone {
                            let dag = Arc::clone(&self.dag);
                            thread::spawn(move || {
                                let mut chain: Vec<Transaction>;
                                let milestone_hash = transaction.get_hash();
                                {
                                    // Verify milestone
                                    match dag.read().unwrap().verify_milestone(transaction) {
                                        Ok(_chain) => {
                                            chain = _chain;
                                        }
                                        Err(_err) => {
                                            // TODO missing transactions
                                            panic!("Missing Transactions: {:?}", _err);
                                        }
                                    }
                                    // Reverse the chain so that the elements closest to the
                                    // milestone are in front
                                    chain = chain.into_iter().rev().collect();
                                }
                                {
                                    // Add chain
                                    let mut dag = dag.write().unwrap();
                                    dag.process_chain(milestone_hash, chain);
                                    if true {
                                        // Sign all existing contracts
                                        // TODO Proper signing
                                        dag.add_pending_signature(MilestoneSignature::new(
                                            hash, 0, 0,
                                        ));
                                        for contract in dag.get_contracts() {
                                            dag.add_pending_signature(MilestoneSignature::new(
                                                hash, contract, 0,
                                            ));
                                        }
                                    }
                                }
                            });
                        }
                        status
                    }
                    Err(TransactionError::Rejected(msg)) => TransactionStatus::Rejected(msg),
                }
            }
            Err(TransactionError::Rejected(msg)) => TransactionStatus::Rejected(msg),
        }
    }

    // Peer functions
    pub fn add_peer(&self, peer: Peer) {
        self.peers.write().unwrap().add_peer(peer);
    }
}
