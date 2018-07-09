use std::sync::RwLock;

use dag::blockdag::BlockDAG;
use dag::transaction::Transaction;
use peermanager::PeerManager;
use util::peer::Peer;
use util::types::TransactionHashes;

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

    pub fn add_transaction(&self, transaction: Transaction) -> bool {
        if self.dag.write().unwrap().add_transaction(&transaction) {
            self.peers.read().unwrap().map_peers(|peer| {
                peer.post_transaction(&transaction)
            });
            true
        }
        else {
            false
        }
    }

    // Peer functions
    pub fn add_peer(&self, peer: Peer) {
        self.peers.write().unwrap().add_peer(peer);
    }
}