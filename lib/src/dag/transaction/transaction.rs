use std::hash::{Hash, Hasher};

use security::hash::hasher::Sha3Hasher;
use security::keys::eddsa::{get_public_key, verify, EdDSAKeyPair};

use super::data::TransactionData;
use super::header::TransactionHeader;
use super::signature::TransactionSignature;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    #[serde(flatten)]
    pub(crate) header: TransactionHeader,
    pub(crate) data: TransactionData,
    pub(crate) signature: TransactionSignature,
}

impl Transaction {
    pub fn new(header: TransactionHeader, data: TransactionData) -> Self {
        Transaction {
            header,
            data,
            signature: TransactionSignature::Unsigned,
        }
    }

    pub fn raw(
        header: TransactionHeader,
        data: TransactionData,
        signature: TransactionSignature,
    ) -> Self {
        Transaction {
            header,
            data,
            signature,
        }
    }

    pub fn get_trunk_hash(&self) -> u64 {
        self.header.trunk_transaction
    }

    pub fn get_branch_hash(&self) -> u64 {
        self.header.branch_transaction
    }

    pub fn get_contract(&self) -> u64 {
        self.header.contract
    }

    pub fn get_trunk_root(&self) -> u64 {
        self.header.trunk_root
    }

    pub fn get_branch_root(&self) -> u64 {
        self.header.branch_root
    }

    pub fn get_merge_root(&self) -> u64 {
        self.header.merge_root
    }

    pub fn get_ancestor_root(&self) -> u64 {
        self.header.ancestor_root
    }

    pub fn get_timestamp(&self) -> u64 {
        self.header.timestamp
    }

    pub fn get_nonce(&self) -> u32 {
        self.header.nonce
    }

    pub fn get_address(&self) -> u64 {
        self.signature.get_address()
    }

    pub fn get_all_refs(&self) -> [u64; 2] {
        let mut refs = Vec::new();
        refs.push(self.get_branch_hash());
        refs.push(self.get_trunk_hash());

        [self.get_branch_hash(), self.get_trunk_hash()]
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn get_data(&self) -> &TransactionData {
        &self.data
    }

    pub fn get_signature(&self) -> &TransactionSignature {
        &self.signature
    }

    pub fn sign_eddsa(&mut self, key: &EdDSAKeyPair) {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        let bytes = &s.finish_bytes();
        let signature = key.sign(bytes);
        self.signature = TransactionSignature::EdDSA {
            public_key: get_public_key(key),
            signature: signature.into(),
        }
    }

    pub fn verify(&self) -> bool {
        let mut s = Sha3Hasher::new();
        self.hash(&mut s);
        let bytes = &s.finish_bytes();
        match self.signature {
            TransactionSignature::Unsigned => false,
            TransactionSignature::EdDSA {
                ref public_key,
                ref signature,
            } => verify(public_key, bytes, signature),
        }
    }
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.header.hash(state);
        self.data.hash(state);
    }
}

impl PartialEq<Transaction> for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.header == other.header && self.data == other.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use security::keys::eddsa::new_key_pair;

    #[test]
    fn test_new_transaction() {
        let branch_hash = 0;
        let trunk_hash = 1;

        let transaction = Transaction::new(
            TransactionHeader::new(branch_hash, trunk_hash, 0, 0, 0, 0, 0, 0, 0),
            TransactionData::Genesis,
        );

        assert_eq!(transaction.get_branch_hash(), branch_hash);
        assert_eq!(transaction.get_trunk_hash(), trunk_hash);
        assert_eq!([branch_hash, trunk_hash], transaction.get_all_refs());
        assert_eq!(0, transaction.get_nonce());
        assert_eq!(8846522215756168669, transaction.get_hash());
    }

    #[test]
    fn test_sign_and_verify_transaction() {
        let key = new_key_pair().unwrap();
        let mut transaction = Transaction::new(
            TransactionHeader::new(0, 0, 0, 0, 0, 0, 0, 0, 0),
            TransactionData::Genesis,
        );
        transaction.sign_eddsa(&key);
        assert!(transaction.verify());
    }

    #[test]
    fn test_serialize() {
        let transaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5, 6, 7, 8),
            TransactionData::Genesis,
        );
        let json_value = json!({
            "branch_transaction": 0,
            "trunk_transaction": 1,
            "contract": 2,
            "trunk_root": 3,
            "branch_root": 4,
            "merge_root": 5,
            "ancestor_root": 6,
            "timestamp": 7,
            "nonce": 8,
            "signature": TransactionSignature::Unsigned,
            "data": TransactionData::Genesis
        });

        assert_eq!(json_value, serde_json::to_value(transaction).unwrap());
    }

    #[test]
    fn test_deserialize() {
        let transaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5, 6, 7, 8),
            TransactionData::Genesis,
        );
        let json_value = json!({
            "branch_transaction": 0,
            "trunk_transaction": 1,
            "contract": 2,
            "trunk_root": 3,
            "branch_root": 4,
            "merge_root": 5,
            "ancestor_root": 6,
            "timestamp": 7,
            "nonce": 8,
            "signature": TransactionSignature::Unsigned,
            "data": TransactionData::Genesis
        });
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check the transaction is identical after serializing and deserializing
        let transaction = Transaction::new(
            TransactionHeader::new(0, 0, 0, 0, 0, 0, 0, 0, 0),
            TransactionData::Genesis,
        );
        let json_value = serde_json::to_value(transaction.clone()).unwrap();
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());

        // Check a signed transaction is identical after serializing and deserializing
        let mut signed_transaction = Transaction::new(
            TransactionHeader::new(0, 0, 0, 0, 0, 0, 0, 0, 0),
            TransactionData::Genesis,
        );
        let key = new_key_pair().unwrap();
        signed_transaction.sign_eddsa(&key);
        let signed_json_value = serde_json::to_value(signed_transaction.clone()).unwrap();
        assert_eq!(
            signed_transaction,
            serde_json::from_value(signed_json_value).unwrap()
        );
    }
}
