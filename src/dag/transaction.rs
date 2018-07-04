use dag::sha3::{Sha3_512,Digest};

use util::{bytes_as_string,epoch_time};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    branch_transaction: String,
    trunk_transaction: String,
    ref_transactions: Vec<String>,
    timestamp: u64,
    nonce: u32,
    transaction_type: u8,
    signature: String,
}

impl Transaction {
    pub fn new(branch_transaction: String, trunk_transaction: String, ref_transactions: Vec<String>,
               timestamp: u64, nonce: u32, transaction_type: u8, signature: String) -> Transaction {
        Transaction {
            branch_transaction: branch_transaction,
            trunk_transaction: trunk_transaction,
            ref_transactions: ref_transactions,
            timestamp: timestamp,
            nonce: nonce,
            transaction_type: transaction_type,
            signature: signature,
        }
    }

    pub fn create(branch_transaction: String, trunk_transaction: String, ref_transactions: Vec<String>, nonce: u32, signature: String) -> Transaction {
        Transaction {
            branch_transaction: branch_transaction,
            trunk_transaction: trunk_transaction,
            ref_transactions: ref_transactions,
            timestamp: epoch_time(),
            nonce: nonce,
            transaction_type: 0,
            signature: signature,
        }
    }

    pub fn get_trunk_hash(&self) -> String {
        self.trunk_transaction.to_owned()
    }

    pub fn get_branch_hash(&self) -> String {
        self.branch_transaction.to_owned()
    }

    pub fn get_ref_hashes(&self) -> Vec<String> {
        self.ref_transactions.clone()
    }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn get_all_refs(&self) -> Vec<String> {
        let mut refs = self.get_ref_hashes();
        refs.push(self.get_branch_hash());
        refs.push(self.get_trunk_hash());

        refs
    }

    pub fn get_hash(&self) -> String {
        let mut value = self.signature.to_owned();
        value.push_str(&self.branch_transaction);
        value.push_str(&self.trunk_transaction);
        value.push_str(&self.ref_transactions.join(""));
        value.push_str(&self.timestamp.to_string());
        let mut hasher = Sha3_512::new();
        hasher.input(value.as_bytes());
        let hash = hasher.result();

        bytes_as_string(&hash.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let branch_hash = String::from("0");
        let trunk_hash = String::from("1");
        let ref_hash = String::from("2");
        let signature = String::from("3");

        let transaction = Transaction::new(branch_hash.clone(),
            trunk_hash.clone(), vec![ref_hash.clone()],
            0, 0, 0, signature.clone());

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
        assert_eq!("72DBDDB94C62BBAF51DEFF730A1ACC60E0081899392E0DB80ED762E8EF91E573C61057A5A238F14C57331835A9439AAE871DADA4FBBA9D4F16AB40773B9BFC4A",
            transaction.get_hash());
    }

    #[test]
    fn test_create_transaction() {
        let branch_hash = String::from("0");
        let trunk_hash = String::from("1");
        let ref_hash = String::from("2");
        let signature = String::from("3");

        let transaction = Transaction::create(branch_hash.clone(),
            trunk_hash.clone(), vec![ref_hash.clone()],
            0, signature.clone());

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!(vec![ref_hash, branch_hash, trunk_hash],
            transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
    }
}