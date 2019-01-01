use std::hash::{Hash,Hasher};
use std::fmt;

use serde::{
    ser::{Serialize, Serializer, SerializeStruct},
    de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess, Unexpected}
};

use security::hash::hasher::Sha3Hasher;
use security::keys::{PrivateKey,PublicKey};
use security::ring::digest::SHA512_256;

use util::epoch_time;

use dag::transaction::data::TransactionData;

#[derive(Clone, Debug)]
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

    pub fn raw(branch_transaction: u64, trunk_transaction: u64,
            ref_transactions: Vec<u64>, contract: u64, timestamp: u64,
            nonce: u32, address: Vec<u8>, signature: Vec<u8>,
            data: TransactionData) -> Self {
        Transaction {
            branch_transaction: branch_transaction,
            trunk_transaction: trunk_transaction,
            ref_transactions: ref_transactions,
            contract: contract,
            timestamp: timestamp,
            nonce: nonce,
            address: address,
            signature: signature,
            data: data,
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

    pub fn get_address(&self) -> &[u8] {
        &self.address
    }

    pub fn get_signature(&self) -> &[u8] {
        &self.signature
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
        self.contract.hash(state);
        self.data.hash(state);
    }
}

impl PartialEq<Transaction> for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.branch_transaction == other.branch_transaction &&
            self.trunk_transaction == other.trunk_transaction &&
            self.ref_transactions == other.ref_transactions &&
            self.timestamp == other.timestamp &&
            self.nonce == other.nonce &&
            self.contract == other.contract &&
            self.data == other.data
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 9 fields in the struct
        let mut state = serializer.serialize_struct("Transaction", 9)?;
        // Serialize fields
        state.serialize_field("branch_transaction", &self.branch_transaction)?;
        state.serialize_field("trunk_transaction", &self.trunk_transaction)?;
        state.serialize_field("ref_transactions", &self.ref_transactions)?;
        state.serialize_field("contract", &self.contract)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("nonce", &self.nonce)?;

        // Serialize address and signature as base64 strings
        state.serialize_field("address",
            &base64::encode_config(&self.address, base64::URL_SAFE))?;
        state.serialize_field("signature",
            &base64::encode_config(&self.signature, base64::URL_SAFE))?;

        state.serialize_field("data", &self.data)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Branch_Transaction,
            Trunk_Transaction,
            Ref_Transactions,
            Contract,
            Timestamp,
            Nonce,
            Address,
            Signature,
            Data,
        }

        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Transaction")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Transaction, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let branch_transaction = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let trunk_transaction = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let ref_transactions = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let contract = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let timestamp = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let nonce = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let address = base64::decode_config(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?, base64::URL_SAFE)
                    .map_err(|_| { de::Error::invalid_value(Unexpected::Str(&"address"), &"valid base64 string")})?;
                let signature = base64::decode_config(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?, base64::URL_SAFE)
                    .map_err(|_| { de::Error::invalid_value(Unexpected::Str(&"signature"), &"valid base64 string")})?;
                let data = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;

                Ok(Transaction::raw(branch_transaction, trunk_transaction, ref_transactions,
                    contract, timestamp, nonce, address, signature, data))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Transaction, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut branch_transaction = None;
                let mut trunk_transaction = None;
                let mut ref_transactions = None;
                let mut contract = None;
                let mut timestamp = None;
                let mut nonce = None;
                let mut address = None;
                let mut signature = None;
                let mut data = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Branch_Transaction => {
                            if branch_transaction.is_some() {
                                return Err(de::Error::duplicate_field("branch_transaction"));
                            }
                            branch_transaction = Some(map.next_value()?);
                        },
                        Field::Trunk_Transaction => {
                            if trunk_transaction.is_some() {
                                return Err(de::Error::duplicate_field("trunk_transaction"));
                            }
                            trunk_transaction = Some(map.next_value()?);
                        },
                        Field::Ref_Transactions => {
                            if ref_transactions.is_some() {
                                return Err(de::Error::duplicate_field("ref_transactions"));
                            }
                            ref_transactions = Some(map.next_value()?);
                        },
                        Field::Contract => {
                            if contract.is_some() {
                                return Err(de::Error::duplicate_field("contract"));
                            }
                            contract = Some(map.next_value()?);
                        },
                        Field::Timestamp => {
                            if timestamp.is_some() {
                                return Err(de::Error::duplicate_field("timestamp"));
                            }
                            timestamp = Some(map.next_value()?);
                        },
                        Field::Nonce => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        },
                        Field::Address => {
                            if address.is_some() {
                                return Err(de::Error::duplicate_field("address"));
                            }
                            address = Some(base64::decode_config(
                                &map.next_value::<String>()?, base64::URL_SAFE)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"address"), &"valid base64 string")})?);
                        },
                        Field::Signature => {
                            if signature.is_some() {
                                return Err(de::Error::duplicate_field("signature"));
                            }
                            signature = Some(base64::decode_config(
                                &map.next_value::<String>()?, base64::URL_SAFE)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"signature"), &"valid base64 string")})?);
                        },
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        },
                    }
                }

                let branch_transaction = branch_transaction.ok_or_else(|| de::Error::duplicate_field("branch_transaction"))?;
                let trunk_transaction = trunk_transaction.ok_or_else(|| de::Error::duplicate_field("trunk_transaction"))?;
                let ref_transactions = ref_transactions.ok_or_else(|| de::Error::duplicate_field("ref_transactions"))?;
                let contract = contract.ok_or_else(|| de::Error::duplicate_field("contract"))?;
                let timestamp = timestamp.ok_or_else(|| de::Error::duplicate_field("timestamp"))?;
                let nonce = nonce.ok_or_else(|| de::Error::duplicate_field("nonce"))?;
                let address = address.ok_or_else(|| de::Error::duplicate_field("address"))?;
                let signature = signature.ok_or_else(|| de::Error::duplicate_field("signature"))?;
                let data = data.ok_or_else(|| de::Error::duplicate_field("data"))?;

                Ok(Transaction::raw(branch_transaction, trunk_transaction, ref_transactions,
                    contract, timestamp, nonce, address, signature, data))
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "branch_transaction",
            "trunk_transaction",
            "ref_transactions",
            "contract",
            "timestamp",
            "nonce",
            "address",
            "signature",
            "data",
        ];
        deserializer.deserialize_struct("Transaction", FIELDS, TransactionVisitor)
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
        assert_eq!(2763323875860498692, transaction.get_hash());
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

    #[test]
    fn test_serialize() {
        let transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let json_value = json!({
            "branch_transaction": 0,
            "trunk_transaction": 1,
            "ref_transactions": vec![2],
            "contract": 3,
            "timestamp": 4,
            "nonce": 5,
            "address": "",
            "signature": base64::encode_config(&vec![0; 8192], base64::URL_SAFE),
            "data": TransactionData::Genesis
        });
        assert_eq!(json_value, serde_json::to_value(transaction).unwrap());
    }

    #[test]
    fn test_deserialize() {
        let transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let json_value = json!({
            "branch_transaction": 0,
            "trunk_transaction": 1,
            "ref_transactions": vec![2],
            "contract": 3,
            "timestamp": 4,
            "nonce": 5,
            "address": "",
            "signature": base64::encode_config(&vec![0; 8192], base64::URL_SAFE),
            "data": TransactionData::Genesis
        });
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check the transaction is identical after serializing and deserializing
        let transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let json_value = serde_json::to_value(transaction.clone()).unwrap();
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());

        // Check a signed transaction is identical after serializing and deserializing
        let mut signed_transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let mut key = PrivateKey::new(&SHA512_256);
        signed_transaction.sign(&mut key);
        let signed_json_value = serde_json::to_value(signed_transaction.clone()).unwrap();
        assert_eq!(signed_transaction, serde_json::from_value(signed_json_value).unwrap());
    }

}