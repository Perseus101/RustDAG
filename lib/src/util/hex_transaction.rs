use dag::transaction::{
    data::TransactionData, header::TransactionHeader, signature::TransactionSignature, Transaction,
};

use super::{u32_as_hex_string, u64_as_hex_string};

mod hex_u32 {
    use super::*;
    use serde::{
        de::{Deserialize, Deserializer, Error},
        ser::Serializer,
    };
    use std::u32;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(key: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&u32_as_hex_string(*key))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|string| {
            u32::from_str_radix(&string, 16).map_err(|err| Error::custom(err.to_string()))
        })
    }
}

mod hex_u64 {
    use super::*;
    use serde::{
        de::{Deserialize, Deserializer, Error},
        ser::Serializer,
    };
    use std::u64;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(key: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&u64_as_hex_string(*key))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|string| {
            u64::from_str_radix(&string, 16).map_err(|err| Error::custom(err.to_string()))
        })
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct HexTransactionHeader {
    #[serde(with = "hex_u64")]
    branch_transaction: u64,
    #[serde(with = "hex_u64")]
    trunk_transaction: u64,
    #[serde(with = "hex_u64")]
    contract: u64,
    #[serde(with = "hex_u64")]
    root: u64,
    #[serde(with = "hex_u64")]
    timestamp: u64,
    #[serde(with = "hex_u32")]
    nonce: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct HexEncodedTransaction {
    #[serde(flatten)]
    header: HexTransactionHeader,
    data: TransactionData,
    signature: TransactionSignature,
}

impl From<TransactionHeader> for HexTransactionHeader {
    fn from(header: TransactionHeader) -> Self {
        HexTransactionHeader {
            branch_transaction: header.branch_transaction,
            trunk_transaction: header.trunk_transaction,
            contract: header.contract,
            root: header.root,
            timestamp: header.timestamp,
            nonce: header.nonce,
        }
    }
}

impl From<HexTransactionHeader> for TransactionHeader {
    fn from(header: HexTransactionHeader) -> Self {
        TransactionHeader {
            branch_transaction: header.branch_transaction,
            trunk_transaction: header.trunk_transaction,
            contract: header.contract,
            root: header.root,
            timestamp: header.timestamp,
            nonce: header.nonce,
        }
    }
}

impl From<Transaction> for HexEncodedTransaction {
    fn from(transaction: Transaction) -> Self {
        HexEncodedTransaction {
            header: transaction.header.into(),
            data: transaction.data,
            signature: transaction.signature,
        }
    }
}

impl From<HexEncodedTransaction> for Transaction {
    fn from(hex: HexEncodedTransaction) -> Transaction {
        Transaction::raw(hex.header.into(), hex.data, hex.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dag::transaction::header::TransactionHeader;
    use security::keys::eddsa::new_key_pair;

    #[test]
    fn test_convert() {
        let transaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5),
            TransactionData::Genesis,
        );
        let hex: HexEncodedTransaction = transaction.clone().into();
        let converted: Transaction = hex.into();
        assert_eq!(transaction, converted);
        assert_eq!(transaction.get_branch_hash(), converted.get_branch_hash());
        assert_eq!(transaction.get_trunk_hash(), converted.get_trunk_hash());
        assert_eq!(transaction.get_contract(), converted.get_contract());
        assert_eq!(transaction.get_nonce(), converted.get_nonce());
        assert_eq!(transaction.get_root(), converted.get_root());
        assert_eq!(transaction.get_address(), converted.get_address());
        assert_eq!(transaction.get_signature(), converted.get_signature());
        assert_eq!(transaction.get_data(), converted.get_data());
    }

    #[test]
    fn test_serialize() {
        let transaction: HexEncodedTransaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5),
            TransactionData::Genesis,
        )
        .into();
        let json_value = json!({
            "branch_transaction": "0000000000000000",
            "trunk_transaction": "0000000000000001",
            "contract": "0000000000000002",
            "root": "0000000000000003",
            "timestamp": "0000000000000004",
            "nonce": "00000005",
            "signature": TransactionSignature::Unsigned,
            "data": TransactionData::Genesis
        });
        assert_eq!(json_value, serde_json::to_value(transaction).unwrap());
    }

    #[test]
    fn test_deserialize() {
        let transaction: HexEncodedTransaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5),
            TransactionData::Genesis,
        )
        .into();
        let json_value = json!({
            "branch_transaction": "0000000000000000",
            "trunk_transaction": "0000000000000001",
            "contract": "0000000000000002",
            "root": "0000000000000003",
            "timestamp": "0000000000000004",
            "nonce": "00000005",
            "signature": TransactionSignature::Unsigned,
            "data": TransactionData::Genesis
        });
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check the transaction is identical after serializing and deserializing
        let transaction: HexEncodedTransaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5),
            TransactionData::Genesis,
        )
        .into();
        let json_value = serde_json::to_value(transaction.clone()).unwrap();
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());

        // Check a signed transaction is identical after serializing and deserializing
        let mut signed_transaction = Transaction::new(
            TransactionHeader::new(0, 1, 2, 3, 4, 5),
            TransactionData::Genesis,
        );
        let key = new_key_pair().unwrap();
        signed_transaction.sign_eddsa(&key);
        let hex_signed_transaction: HexEncodedTransaction = signed_transaction.into();
        let signed_json_value = serde_json::to_value(hex_signed_transaction.clone()).unwrap();
        assert_eq!(
            hex_signed_transaction,
            serde_json::from_value(signed_json_value).unwrap()
        );
    }
}
