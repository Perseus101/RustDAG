extern crate restson;
use self::restson::{RestClient,RestPath,Error};

use dag::transaction::Transaction;
use client::types::{TransactionHashes,ProcessStatus};

impl RestPath<()> for TransactionHashes {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("tips")) }
}

enum TransactionRequest {
    GET(u64),
    POST()
}

impl RestPath<TransactionRequest> for Transaction {
    fn get_path(param: TransactionRequest) -> Result<String,Error> {
        match param {
            TransactionRequest::GET(hash) => Ok(format!("transaction/get/{}", hash)),
            TransactionRequest::POST() => Ok(String::from("transaction"))
        }
    }
}

#[derive(Deserialize)]
pub struct Peer {
    client_url: String
}

impl Peer {
    pub fn new(client_url: String) -> Peer {
        Peer {
            client_url: client_url
        }
    }

    pub fn get_transaction(&self, hash: u64) -> Option<Transaction> {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(TransactionRequest::GET(hash)).ok()
    }

    pub fn post_transaction(&self, transaction: &Transaction) -> ProcessStatus {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.post_capture(TransactionRequest::POST(), transaction).unwrap()
    }

    pub fn get_tips(&self) -> TransactionHashes {
        let mut client = RestClient::new(&self.client_url).unwrap();
        client.get(()).unwrap()
    }
}