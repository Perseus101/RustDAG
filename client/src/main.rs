use security::ring::digest::SHA512_256;

extern crate rustdag_lib;

use rustdag_lib::{util, security, dag};

use dag::transaction::{Transaction, data::TransactionData};
use dag::contract::Contract;
use dag::contract::source::{ContractSource, function::ContractFunction, op::ContractOp};

use util::peer::Peer;
use util::types::TransactionStatus;
use security::hash::proof::proof_of_work;
use security::keys::PrivateKey;

fn main() {
    let server = Peer::new(String::from("http://localhost:4200"));

    // Generate contract
    let contract_src = ContractSource::new(vec![
        ContractFunction::new(vec![ContractOp::AddConst((1, 0, 0))], 0, 0)
    ], 1);

    let mut contract_id = 0;
    let tip_hashes = server.get_tips();
    if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash) {
        if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
            let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

            let mut pk = PrivateKey::new(&SHA512_256);

            let mut transaction = Transaction::create(
                tip_hashes.branch_hash, tip_hashes.trunk_hash, vec![],
                0, nonce, TransactionData::GenContract(contract_src.clone())
            );

            transaction.sign(&mut pk);

            contract_id = transaction.get_hash();

            if server.post_transaction(&transaction) == TransactionStatus::Rejected {
                panic!("Contract rejected");
            }
        }
    }

    // Execute the contract repeatedly
    let mut contract: Contract = From::from(contract_src);

    loop {
        let tip_hashes = server.get_tips();
        if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash) {
            if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
                let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

                if let Ok(result) = contract.exec(0, vec![]) {
                    if let Err(err) = contract.apply(result.clone()) {
                        println!("Error running contract: {:?}", err);
                        continue;
                    }
                    let mut pk = PrivateKey::new(&SHA512_256);

                    let mut transaction = Transaction::create(
                        tip_hashes.branch_hash, tip_hashes.trunk_hash, vec![],
                        contract_id, nonce, TransactionData::ExecContract(result)
                    );

                    transaction.sign(&mut pk);

                    println!("Transaction {}: {}", transaction.get_hash(), contract.get_state()[0]);
                    if server.post_transaction(&transaction) == TransactionStatus::Milestone {
                        println!("Milestone");
                    }
                }
            }
        }
    }
}
