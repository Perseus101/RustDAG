use std::hash::{Hash,Hasher};

use security::hash::hasher::Sha3Hasher;

use util::epoch_time;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    branch_transaction: u64,
    trunk_transaction: u64,
    ref_transactions: Vec<u64>,
    timestamp: u64,
    nonce: u32,
    transaction_type: u8,
    signature: String,
}

impl Transaction {
    pub fn new(branch_transaction: u64, trunk_transaction: u64, ref_transactions: Vec<u64>,
               timestamp: u64, nonce: u32, transaction_type: u8) -> Transaction {
        Transaction {
            branch_transaction: branch_transaction,
            trunk_transaction: trunk_transaction,
            ref_transactions: ref_transactions,
            timestamp: timestamp,
            nonce: nonce,
            transaction_type: transaction_type,
            signature: String::from(""),
        }
    }

    pub fn create(branch_transaction: u64, trunk_transaction: u64, ref_transactions: Vec<u64>, nonce: u32) -> Transaction {
        Transaction {
            branch_transaction: branch_transaction,
            trunk_transaction: trunk_transaction,
            ref_transactions: ref_transactions,
            timestamp: epoch_time(),
            nonce: nonce,
            transaction_type: 0,
            signature: String::from(""),
        }
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

    pub fn get_hash(&self) -> u64 {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.branch_transaction.hash(state);
        self.trunk_transaction.hash(state);
        self.ref_transactions.hash(state);
        self.timestamp.hash(state);
        self.nonce.hash(state);
        self.transaction_type.hash(state);
        self.signature.hash(state);
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

        let transaction = Transaction::new(branch_hash,
            trunk_hash, vec![ref_hash], 0, 0, 0);

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
        assert_eq!(15088500869469674164, transaction.get_hash());
    }

    #[test]
    fn test_create_transaction() {
        let branch_hash = 0;
        let trunk_hash = 1;
        let ref_hash = 2;

        let transaction = Transaction::create(branch_hash.clone(),
            trunk_hash.clone(), vec![ref_hash.clone()], 0);

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
    }
}