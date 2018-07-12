use std::sync::RwLock;

use dag::blockdag::BlockDAG;
use dag::transaction::Transaction;
use peermanager::PeerManager;
use util::peer::Peer;
use util::types::{TransactionHashes,TransactionStatus};

pub struct DAGManager {
    dag: RwLock<BlockDAG>,
    peers: RwLock<PeerManager>,
}

impl Default for DAGManager {
	fn default() -> Self {
        DAGManager {
            dag: RwLock::from(BlockDAG::default()),
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

    pub fn add_transaction(&self, transaction: Transaction) -> TransactionStatus {
        let status = self.dag.write().unwrap().add_transaction(&transaction);
        if status != TransactionStatus::Rejected {
            self.peers.read().unwrap().map_peers(|peer| {
                peer.post_transaction(&transaction)
            });
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