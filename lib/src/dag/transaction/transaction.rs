use std::hash::{Hash,Hasher};

use security::hash::hasher::Sha3Hasher;
use security::keys::{PrivateKey,PublicKey};
use security::ring::digest::SHA512_256;

use util::epoch_time;

use dag::transaction::data::TransactionData;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    branch_transaction: u64,
    trunk_transaction: u64,
    ref_transactions: Vec<u64>,
    contract: u64,
    timestamp: u64,
    nonce: u32,
    address: Vec<u8>,
    signature: Vec<u8>,
    data: TransactionData,
}

impl Transaction {
    pub fn new(branch_transaction: u64, trunk_transaction: u64, ref_transactions: Vec<u64>,
               contract: u64, timestamp: u64, nonce: u32, data: TransactionData) -> Self {
        Transaction {
            branch_transaction,
            trunk_transaction,
            ref_transactions,
            contract,
            timestamp,
            nonce,
            address: Vec::new(),
            signature: vec![0; 8192],
            data,
        }
    }

    pub fn create(branch_transaction: u64, trunk_transaction: u64, ref_transactions: Vec<u64>,
                  contract: u64, nonce: u32, data: TransactionData) -> Self {
        Transaction::new(
            branch_transaction,
            trunk_transaction,
            ref_transactions,
            contract,
            epoch_time(),
            nonce,
            data
        )
    }

    pub fn get_trunk_hash(&self) -> u64 {
        self.trunk_transaction
    }

    pub fn get_branch_hash(&self) -> u64 {
        self.branch_transaction
    }

    pub fn get_ref_hashes(&self) -> Vec<u64> {
        self.ref_transactions.clone()
    }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn get_all_refs(&self) -> Vec<u64> {
        let mut refs = self.get_ref_hashes();
        refs.push(self.get_branch_hash());
        refs.push(self.get_trunk_hash());

        refs
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn get_contract(&self) -> u64 {
        self.contract
    }

    pub fn get_data(&self) -> &TransactionData {
        &self.data
    }

    pub fn sign(&mut self, key: &mut PrivateKey) {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        let bytes = &s.finish_bytes();
        if let Ok(signature) = key.sign(bytes) {
            // The signature is composed of 256 fragments, which are each arrays of 32 bytes
            for (sig_frag, i) in signature.iter().zip(0..) {
                self.signature[i*32..(i+1)*32].copy_from_slice(sig_frag);
            }
            self.address = key.public_key().to_bytes()
        }
    }

    pub fn verify(&self) -> bool {
        if let Some(key) = PublicKey::from_vec(self.address.clone(), &SHA512_256) {
            let mut s = Sha3Hasher::new();
            self.hash(&mut s);
            let bytes = &s.finish_bytes();
            let mut signature = vec![vec![0; 32]; 256];
            for i in 0..256 {
                signature[i].copy_from_slice(&self.signature[i*32..(i+1)*32]);
            }
            return key.verify_signature(&signature, bytes);
        }
        false
    }
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.branch_transaction.hash(state);
        self.trunk_transaction.hash(state);
        self.ref_transactions.hash(state);
        self.timestamp.hash(state);
        self.nonce.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let branch_hash = 0;
        let trunk_hash = 1;
        let ref_hash = 2;

        let transaction = Transaction::new(branch_hash, trunk_hash,
            vec![ref_hash], 0, 0, 0, TransactionData::Genesis);

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
        assert_eq!(7216540755162860552, transaction.get_hash());
    }

    #[test]
    fn test_create_transaction() {
        let branch_hash = 0;
        let trunk_hash = 1;
        let ref_hash = 2;

        let transaction = Transaction::create(branch_hash, trunk_hash,
            vec![ref_hash], 0, 0, TransactionData::Genesis);

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
    }

    #[test]
    fn test_sign_and_verify_transaction() {
        let mut key = PrivateKey::new(&SHA512_256);
        let mut transaction = Transaction::create(0, 0, vec![], 0, 0, TransactionData::Genesis);
        transaction.sign(&mut key);
        assert!(transaction.verify());
    }
}