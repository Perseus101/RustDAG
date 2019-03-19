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
    fn get_path(hash: u64) -> Result<String, Error> {
        Ok(format!("contract/{}", hash))
    }
}

impl RestPath<u64> for Node<ContractValue> {
    fn get_path(hash: u64) -> Result<String, Error> {
        Ok(format!("node/{}", hash))
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

    pub fn get_transaction(&self, hash: u64) -> Result<Transaction, Error> {
        let mut client = RestClient::new(&self.client_url)?;
        client.get(TransactionRequest::GET(hash))
    }

    pub fn post_transaction(&self, transaction: &Transaction) -> Result<TransactionStatus, Error> {
        let mut client = RestClient::new(&self.client_url)?;
        client.post_capture(TransactionRequest::POST(), transaction)
    }

    pub fn get_tips(&self) -> TransactionHashes {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(()).unwrap()
    }

    pub fn get_contract(&self, hash: u64) -> Result<Contract, Error> {
        let mut client = RestClient::new(&self.client_url)?;
        client.get(hash)
    }

    pub fn get_mpt_node(&self, hash: u64) -> Result<Node<ContractValue>, Error> {
        let mut client = RestClient::new(&self.client_url)?;
        client.get(hash)
    }
}

impl Map<u64, Transaction> for TransactionPeer {
    fn get(&self, k: &u64) -> MapResult<OOB<Transaction>> {
        match self.0.get_transaction(*k) {
            Ok(transaction) => Ok(OOB::Owned(transaction)),
            Err(_) => Err(MapError::LookupError)
        }
    }

    fn set(&mut self, _: u64, v: Transaction) -> MapResult<()> {
        let _status =
            self.0.post_transaction(&v).map_err(|_| { MapError::LookupError })?;
        // TODO check status
        Ok(())
    }
}

impl Map<u64, Contract> for ContractPeer {
    fn get(&self, k: &u64) -> MapResult<OOB<Contract>> {
        match self.0.get_contract(*k) {
            Ok(contract) => Ok(OOB::Owned(contract)),
            Err(_) => Err(MapError::LookupError)
        }
    }

    fn set(&mut self, _: u64, _: Contract) -> MapResult<()> {
        unimplemented!("Cannot post contracts");
    }
}

impl Map<u64, Node<ContractValue>> for MPTNodePeer {
    fn get(&self, k: &u64) -> MapResult<OOB<Node<ContractValue>>> {
        // Get from the local nodes
        // If no nodes exist, then check the remote nodes
        self.nodes.get(k).map_or_else(|| {
                self.peer.get_mpt_node(*k)
                    .map_err(|_| { MapError::LookupError })
                    .map(|node| { OOB::Owned(node) })
            },
            |node| {
                Ok(OOB::Borrowed(node))
            }
        )
    }
    fn set(&mut self, k: u64, v: Node<ContractValue>) -> MapResult<()> {
        self.nodes.insert(k, v);
        Ok(())
    }
}