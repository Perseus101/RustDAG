use std::fs::File;
use std::io::Read;

use rustdag_lib::{
    util::{types::TransactionStatus, peer::Peer},
    security::{
        hash::proof::proof_of_work,
        keys::PrivateKey,
        ring::digest::SHA512_256
    },
    dag::{
        transaction::{Transaction, data::TransactionData},
        contract::source::ContractSource,
    }
};

pub fn deploy_contract(server_url: String, filename: String) -> u64 {
    let server = Peer::new(server_url);
    let blockdag = server.clone().into_remote_blockdag();
    let root = blockdag.get_mpt_default_root();

    // Load contract
    let mut file = File::open(filename).expect("Could not open file");
    let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut buf)
        .expect("Could not read test file");
    let contract_src = ContractSource::new(&buf);

    let tip_hashes = server.get_tips();
    let trunk = server.get_transaction(tip_hashes.trunk_hash).unwrap();
    let branch = server.get_transaction(tip_hashes.branch_hash).unwrap();
    let trunk_nonce = proof_of_work(trunk.get_nonce(), branch.get_nonce());

    let mut pk = PrivateKey::new(&SHA512_256);

    let mut transaction = Transaction::create(
        tip_hashes.branch_hash,
        tip_hashes.trunk_hash,
        vec![],
        0,
        trunk_nonce,
        root,
        TransactionData::GenContract(contract_src.clone()),
    );

    transaction.sign(&mut pk);

    let contract_id = transaction.get_hash();

    if let Ok(TransactionStatus::Rejected(msg)) = server.post_transaction(&transaction) {
        panic!("Contract rejected: {}", msg);
    }
    contract_id
}