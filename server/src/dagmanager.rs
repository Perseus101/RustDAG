use std::sync::RwLock;
use std::thread;
use std::sync::Arc;

use dag::{
    blockdag::BlockDAG,
    transaction::Transaction,
    milestone::pending::MilestoneSignature
};
use peermanager::PeerManager;
use util::peer::Peer;
use util::types::{TransactionHashes,TransactionStatus};

pub struct DAGManager {
    dag: Arc<RwLock<BlockDAG>>,
    peers: RwLock<PeerManager>,
}

impl Default for DAGManager {
	fn default() -> Self {
        DAGManager {
            dag: Arc::new(RwLock::from(BlockDAG::default())),
            peers: RwLock::from(PeerManager::new()),
        }
    }
}

impl DAGManager {
    pub fn get_tips(&self) -> TransactionHashes {
        self.dag.read().unwrap().get_tips()
    }

    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        self.dag.read().unwrap().get_transaction(hash)
    }

    pub fn get_transaction_status(&self, hash: u64) -> TransactionStatus {
        self.dag.read().unwrap().get_confirmation_status(hash)
    }

    pub fn add_transaction(&self, transaction: Transaction) -> TransactionStatus {
        let hash = transaction.get_hash();
        let branch_hash = transaction.get_branch_hash();
        let trunk_hash = transaction.get_trunk_hash();
        {
            // Ignore any already known transactions
            let current_status = self.dag.read().unwrap()
                .get_confirmation_status(hash);
            if current_status != TransactionStatus::Rejected {
                return current_status;
            }
        }

        let status: TransactionStatus;
        {
            // Scope the dag write so that it is opened and closed quickly
            status = self.dag.write().unwrap().add_transaction(&transaction);
        }
        if status != TransactionStatus::Rejected {
            self.peers.read().unwrap().map_peers(|peer| {
                peer.post_transaction(&transaction)
            });
            if status == TransactionStatus::Milestone {
                let dag = Arc::clone(&self.dag);
                thread::spawn(move || {
                    let mut chain: Vec<Transaction>;
                    {
                        // Verify milestone
                        match dag.read().unwrap().verify_milestone(transaction) {
                            Ok(_chain) => {
                                chain = _chain;
                            },
                            Err(_err) => {
                                // TODO missing transactions
                                panic!("Missing Transactions: {:?}", _err);
                            }
                        }
                        // Reverse the chain so that the elements closest to the
                        // milestone are in front
                        chain = chain.into_iter().rev().collect();
                        println!("Hash {:?}: {:?} - {:?}", hash, trunk_hash, branch_hash);
                        for chain_item in chain.iter() {
                            println!("Hash {:?}: {:?} - {:?}", chain_item.get_hash(), chain_item.get_trunk_hash(), chain_item.get_branch_hash());
                        }
                    }
                    {
                        // Add chain
                        let mut dag = dag.write().unwrap();
                        dag.process_chain(chain);
                        if true {
                            // Sign all existing contracts
                            // TODO Proper signing
                            dag.add_pending_signature(MilestoneSignature::new(hash, 0, 0));
                            for contract in dag.get_contracts() {
                                dag.add_pending_signature(MilestoneSignature::new(hash, contract, 0));
                            }
                            println!("Confirmation status: {:?} - {:?}\n\n\n\n\n", hash, dag.get_confirmation_status(hash))
                        }
                    }
                });
            }
            status
        }
        else {
            TransactionStatus::Rejected
        }
    }

    // Peer functions
    pub fn add_peer(&self, peer: Peer) {
        self.peers.write().unwrap().add_peer(peer);
    }
}