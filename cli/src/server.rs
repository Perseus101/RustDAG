use std::fs::File;
use std::io::Read;

use rustdag_lib::{
    dag::{
        blockdag::BlockDAG,
        contract::source::ContractSource,
        transaction::{data::TransactionData, Transaction},
    },
    security::{hash::proof::proof_of_work, keys::PrivateKey, ring::digest::SHA512_256},
    util::peer::{types::TransactionStatus, ContractPeer, MPTNodePeer, Peer, TransactionPeer},
};

pub struct Server {
    peer: Peer,
    blockdag: BlockDAG<MPTNodePeer, TransactionPeer, ContractPeer>,
}

impl Server {
    pub fn new<'a>(server_url: &'a str) -> Self {
        let peer = Peer::new(String::from(server_url));
        let blockdag = peer.clone().into_remote_blockdag();
        Server { peer, blockdag }
    }

    pub fn run_contract(&self, contract: u64, function: String, args: Vec<String>) {
        println!("Contract: {}", contract);
        println!("Function: {}", function);
        println!("Arguments: {:?}", args);
    }

    pub fn deploy_contract<'a>(&self, filename: &str) -> u64 {
        let root = self.blockdag.get_mpt_default_root();

        // Load contract
        let mut file = File::open(filename).expect("Could not open file");
        let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf)
            .expect("Could not read test file");
        let contract_src = ContractSource::new(&buf);

        let tip_hashes = self.server.get_tips();
        let trunk = self.server.get_transaction(tip_hashes.trunk_hash).unwrap();
        let branch = self.server.get_transaction(tip_hashes.branch_hash).unwrap();
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
}
