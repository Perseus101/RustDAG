use security::ring::digest::SHA512_256;

extern crate rustdag_lib;

use rustdag_lib::{util, security, dag};

use dag::transaction::Transaction;
use util::peer::Peer;
use util::types::TransactionStatus;
use security::hash::proof::proof_of_work;
use security::keys::PrivateKey;

fn main() {
    let server = Peer::new(String::from("http://localhost:4200"));

    loop {
        let tip_hashes = server.get_tips();
        if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash) {
            if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
                let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

                let mut pk = PrivateKey::new(&SHA512_256);

                let mut transaction = Transaction::create(
                    tip_hashes.branch_hash, tip_hashes.trunk_hash, vec![], nonce
                );

                transaction.sign(&mut pk);

                println!("Transaction {}", transaction.get_hash());
                if server.post_transaction(&transaction) == TransactionStatus::Milestone {
                    println!("Milestone");
                }
            }
        }
    }
}
