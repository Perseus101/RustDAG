use std::sync::RwLock;

use dag::blockdag::BlockDAG;
use dag::transaction::Transaction;

use server::peermanager::PeerManager;
use server::peer::Peer;

use client::types::TransactionHashes;

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
    pub fn get_tips(&self) -> Vec<Transaction> {
        self.dag.read().unwrap().get_tips()
            .iter().map(|transaction| (*transaction).clone()).collect()
    }

    pub fn select_tips(&self) -> TransactionHashes {
        let transactions = self.get_tips();
        TransactionHashes::new(transactions[0].get_hash(), transactions[1].get_hash())
    }

    pub fn get_transaction(&self, hash: String) -> Option<Transaction> {
        self.dag.read().unwrap().get_transaction(&hash)
    }

    pub fn add_transaction(&self, transaction: Transaction) -> bool {
        self.dag.write().unwrap().add_transaction(transaction)
    }

    // Peer functions
    pub fn add_peer(&self, peer: Peer) {
        self.peers.write().unwrap().add_peer(peer);
    }
}