use std::collections::HashMap;

extern crate restson;
use self::restson::{RestClient, RestPath, Error};

use dag::{
    blockdag::BlockDAG,
    storage::map::{Map, OOB, MapError, MapResult},
    storage::mpt::{node::Node},
    transaction::Transaction,
    contract::{Contract, ContractValue},
};

use util::types::{TransactionHashes, TransactionStatus};

impl RestPath<()> for TransactionHashes {
    fn get_path(_: ()) -> Result<String, Error> { Ok(String::from("tips")) }
}

enum TransactionRequest {
    GET(u64),
    POST()
}

impl RestPath<TransactionRequest> for Transaction {
    fn get_path(param: TransactionRequest) -> Result<String, Error> {
        match param {
            TransactionRequest::GET(hash) => Ok(format!("transaction/{}", hash)),
            TransactionRequest::POST() => Ok(String::from("transaction"))
        }
    }
}

impl RestPath<u64> for Contract {
    fn get_path(hash: u64)  -> Result<String, Error> {
        Ok(format!("contract/{}", hash))
    }
}

#[derive(Clone, Deserialize)]
pub struct Peer {
    client_url: String
}

pub struct TransactionPeer(Peer);
pub struct ContractPeer(Peer);
pub struct MPTNodePeer {
    peer: Peer,
    nodes: HashMap<u64, Node<ContractValue>>
}

impl Peer {
    pub fn new(client_url: String) -> Peer {
        Peer {
            client_url
        }
    }

    pub fn into_remote_blockdag(self) -> BlockDAG<MPTNodePeer, TransactionPeer, ContractPeer> {
        let t = TransactionPeer(self.clone());
        let c = ContractPeer(self.clone());
        let m = MPTNodePeer { peer: self, nodes: HashMap::new() };

        BlockDAG::new(t, c, m)
    }

    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(TransactionRequest::GET(hash)).ok()
    }

    pub fn post_transaction(&self, transaction: &Transaction) -> TransactionStatus {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.post_capture(TransactionRequest::POST(), transaction).unwrap()
    }

    pub fn get_tips(&self) -> TransactionHashes {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(()).unwrap()
    }

    pub fn get_contract(&self, hash: u64) -> Option<Contract> {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(hash).ok()
    }
}

impl Map<u64, Transaction> for TransactionPeer {
    fn get<>(& self, k: &u64) -> MapResult<OOB<Transaction>> {
        match self.0.get_transaction(*k) {
            // TODO Some(transaction) => Ok(OOB::Owned(transaction)),
            Some(_) => Err(MapError::NotFound),
            None => Err(MapError::LookupError)
        }
    }

    fn set(&mut self, _: u64, v: Transaction) -> MapResult<()> {
        self.0.post_transaction(&v);
        Ok(())
    }
}

impl Map<u64, Contract> for ContractPeer {
    fn get(&self, k: &u64) -> MapResult<OOB<Contract>> {
        match self.0.get_contract(*k) {
            // TODO Some(contract) => Ok(OOB::Owned(contract)),
            Some(_) => Err(MapError::NotFound),
            None => Err(MapError::LookupError)
        }
    }

    fn set(&mut self, _: u64, _: Contract) -> MapResult<()> {
        unimplemented!("Cannot post contracts");
    }
}

impl Map<u64, Node<ContractValue>> for MPTNodePeer {
    fn get(&self, _k: &u64) -> MapResult<OOB<Node<ContractValue>>> {
        Err(MapError::LookupError)
    }
    fn set(&mut self, _k: u64, _v: Node<ContractValue>) -> MapResult<()> {
        Err(MapError::LookupError)
    }
}