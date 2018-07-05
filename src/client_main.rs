#![feature(test)]

#[macro_use] extern crate serde_derive;

use std::io;

mod util;
mod security;
mod dag;

#[allow(dead_code)]
mod server;
mod client;

use dag::transaction::Transaction;
use server::peer::Peer;
use security::hash::proof::proof_of_work;

fn main() {
    let server = Peer::new(String::from("http://localhost:4200"));

    let tip_hashes = server.get_tips();
    if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash.clone()) {
        if let Some(branch) = server.get_transaction(tip_hashes.branch_hash.clone()) {
            let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

            let mut signature = String::new();

            io::stdin().read_line(&mut signature).expect("Failed to read line");

            let transaction = Transaction::create(
                tip_hashes.branch_hash, tip_hashes.trunk_hash,
                vec![], nonce, signature.trim().to_string()
            );
            println!("{:?}", transaction);
            server.post_transaction(&transaction);
        }
    }
}
