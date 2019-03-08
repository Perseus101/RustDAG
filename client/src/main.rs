use std::fs::File;
use std::io::Read;

extern crate rustdag_lib;

use rustdag_lib::{util, security, dag};

use dag::transaction::{Transaction, data::TransactionData};
use dag::contract::{Contract, ContractValue};
use dag::contract::source::ContractSource;

use util::peer::Peer;
use util::types::TransactionStatus;
use security::ring::digest::SHA512_256;
use security::hash::proof::proof_of_work;
use security::keys::PrivateKey;

fn confirm_transactions(server: &Peer) {
    print!("Confirming");
    loop {
        let tip_hashes = server.get_tips();
        if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash) {
            if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
                let nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

                let mut pk = PrivateKey::new(&SHA512_256);

                let mut transaction = Transaction::create(
                    tip_hashes.branch_hash, tip_hashes.trunk_hash, vec![],
                    0, nonce, 0, TransactionData::Empty
                );

                transaction.sign(&mut pk);

                if server.post_transaction(&transaction) == TransactionStatus::Milestone {
                    println!("\nConfirmed.");
                    break;
                }
                print!(".");
            }
        }
    }
}

fn main() {
    let server = Peer::new(String::from("http://localhost:4200"));

    // Load contract
    let mut file = File::open("test.wasm").expect("Could not open test file");
    let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut buf).expect("Could not read test file");
    let contract_src = ContractSource::new(&buf);

    let mut contract_id = 0;
    let mut trunk_nonce = 0;
    let tip_hashes = server.get_tips();
    if let Some(trunk) = server.get_transaction(tip_hashes.trunk_hash) {
        if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
            trunk_nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

            let mut pk = PrivateKey::new(&SHA512_256);

            let mut transaction = Transaction::create(
                tip_hashes.branch_hash, tip_hashes.trunk_hash, vec![], 0,
                trunk_nonce, 0, TransactionData::GenContract(contract_src.clone())
            );

            transaction.sign(&mut pk);

            contract_id = transaction.get_hash();

            if let TransactionStatus::Rejected(_) = server.post_transaction(&transaction) {
                panic!("Contract rejected");
            }
        }
    }

    // Send empty transactions until the contract is confirmed
    println!("Contract {} sent, awaiting confirmation...", contract_id);
    confirm_transactions(&server);

    let mut trunk_hash = contract_id;
    // Execute the contract grant function
    let mut contract: Contract = Contract::new(contract_src, contract_id).expect("Failed to create contract");
    for data in [
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(1), ContractValue::U64(101)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(2), ContractValue::U64(102)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(3), ContractValue::U64(103)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(4), ContractValue::U64(104)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(5), ContractValue::U64(105)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(6), ContractValue::U64(106)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(7), ContractValue::U64(107)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(8), ContractValue::U64(108)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(9), ContractValue::U64(109)]),
            TransactionData::ExecContract("grant".into(), vec![ContractValue::U64(10), ContractValue::U64(1000)]),
            ].iter() {
        let tip_hashes = server.get_tips();
        if let Some(branch) = server.get_transaction(tip_hashes.branch_hash) {
            trunk_nonce = proof_of_work(trunk_nonce, branch.get_nonce());
            let mut pk = PrivateKey::new(&SHA512_256);
            let mut transaction = Transaction::create(
                tip_hashes.branch_hash, trunk_hash, vec![],
                contract_id, trunk_nonce, 0, data.clone()
            );
            transaction.sign(&mut pk);
            trunk_hash = transaction.get_hash();
            print!("Transaction {}: ", transaction.get_hash());
            // if let TransactionData::ExecContract(func_name, args) = data {
            //     match contract.exec(func_name, &args) {
            //         Err(err) => {
            //             println!("Execution error: {:?}", err);
            //         },
            //         Ok((ret, state)) => {
            //             println!("{:?}", ret);
            //             contract.writeback(state).expect("Error writing out cache");
            //         }
            //     }
            // }
            match server.post_transaction(&transaction) {
                TransactionStatus::Milestone => println!("Milestone"),
                TransactionStatus::Rejected(message) => println!("Rejected: {:?}", message),
                _ => {}
            }
        }
    }
    confirm_transactions(&server);
}
