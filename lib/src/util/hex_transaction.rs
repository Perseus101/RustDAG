use std::fmt;
use std::{u64, u32};
use std::num::ParseIntError;

use serde::{
    ser::{Serialize, Serializer, SerializeStruct},
    de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess, Unexpected}
};

use super::{u64_as_hex_string, u32_as_hex_string};

use dag::transaction::{Transaction, data::TransactionData};

#[derive(Clone, PartialEq, Debug)]
pub struct HexEncodedTransaction {
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

impl From<Transaction> for HexEncodedTransaction {
    fn from(transaction: Transaction) -> Self {
        HexEncodedTransaction {
            branch_transaction: transaction.get_branch_hash(),
            trunk_transaction: transaction.get_trunk_hash(),
            ref_transactions: transaction.get_ref_hashes(),
            contract: transaction.get_contract(),
            timestamp: transaction.get_timestamp(),
            nonce: transaction.get_nonce(),
            address: transaction.get_address().to_vec(),
            signature: transaction.get_signature().to_vec(),
            data: transaction.get_data().clone(),
        }
    }
}

impl From<HexEncodedTransaction> for Transaction {
    fn from(hex: HexEncodedTransaction) -> Transaction {
        Transaction::raw(hex.branch_transaction, hex.trunk_transaction,
            hex.ref_transactions, hex.contract, hex.timestamp, hex.nonce,
            hex.address, hex.signature, hex.data)
    }
}

impl Serialize for HexEncodedTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 9 fields in the struct
        let mut state = serializer.serialize_struct("HexEncodedTransaction", 9)?;
        // Serialize fields
        // Convert integer fields to hex strings
        state.serialize_field("branch_transaction", &u64_as_hex_string(self.branch_transaction))?;
        state.serialize_field("trunk_transaction", &u64_as_hex_string(self.trunk_transaction))?;
        let refs: Vec<String> = self.ref_transactions.iter().map(|val| u64_as_hex_string(*val)).collect();
        state.serialize_field("ref_transactions", &refs)?;
        state.serialize_field("contract", &u64_as_hex_string(self.contract))?;
        state.serialize_field("timestamp", &u64_as_hex_string(self.timestamp))?;
        state.serialize_field("nonce", &u32_as_hex_string(self.nonce))?;
        state.serialize_field("address", &base64::encode_config(&self.address, base64::URL_SAFE))?;
        state.serialize_field("signature", &base64::encode_config(&self.signature, base64::URL_SAFE))?;
        state.serialize_field("data", &self.data)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for HexEncodedTransaction {
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
            type Value = HexEncodedTransaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct HexEncodedTransaction")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<HexEncodedTransaction, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let branch_transaction = u64::from_str_radix(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?, 16)
                    .map_err(|_| {de::Error::invalid_value(Unexpected::Str(&"branch_transaction"), &"valid hex string")})?;

                let trunk_transaction = u64::from_str_radix(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?, 16)
                    .map_err(|_| {de::Error::invalid_value(Unexpected::Str(&"trunk_transaction"), &"valid hex string")})?;

                let ref_transaction_results: Vec<Result<u64, ParseIntError>> = seq.next_element::<Vec<String>>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?.iter()
                        .map(|val| {u64::from_str_radix(&val, 16)}).collect();
                let mut ref_transactions: Vec<u64> = Vec::with_capacity(ref_transaction_results.len());
                for item in ref_transaction_results.into_iter() {
                    match item {
                        Err(_) => return Err(de::Error::invalid_value(
                            Unexpected::Str("ref_transactions"), &"valid hex string")),
                        Ok(valid_item) => ref_transactions.push(valid_item)
                    }
                }

                let contract = u64::from_str_radix(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?, 16)
                    .map_err(|_| {de::Error::invalid_value(Unexpected::Str(&"contract"), &"valid hex string")})?;

                let timestamp = u64::from_str_radix(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?, 16)
                    .map_err(|_| {de::Error::invalid_value(Unexpected::Str(&"timestamp"), &"valid hex string")})?;

                let nonce = u32::from_str_radix(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?, 16)
                    .map_err(|_| {de::Error::invalid_value(Unexpected::Str(&"nonce"), &"valid hex string")})?;

                let address = base64::decode_config(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?, base64::URL_SAFE)
                    .map_err(|_| { de::Error::invalid_value(Unexpected::Str(&"address"), &"valid base64 string")})?;

                let signature = base64::decode_config(&seq.next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?, base64::URL_SAFE)
                    .map_err(|_| { de::Error::invalid_value(Unexpected::Str(&"signature"), &"valid base64 string")})?;

                let data = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;

                Ok(HexEncodedTransaction{
                    branch_transaction: branch_transaction,
                    trunk_transaction: trunk_transaction,
                    ref_transactions: ref_transactions,
                    contract: contract,
                    timestamp: timestamp,
                    nonce: nonce,
                    address: address,
                    signature: signature,
                    data: data
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<HexEncodedTransaction, V::Error>
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
                            branch_transaction = Some(u64::from_str_radix(
                                &map.next_value::<String>()?, 16)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"branch_transaction"), &"valid hex string")})?);
                        },
                        Field::Trunk_Transaction => {
                            if trunk_transaction.is_some() {
                                return Err(de::Error::duplicate_field("trunk_transaction"));
                            }
                            trunk_transaction = Some(u64::from_str_radix(
                                &map.next_value::<String>()?, 16)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"trunk_transaction"), &"valid hex string")})?);
                        },
                        Field::Ref_Transactions => {
                            if ref_transactions.is_some() {
                                return Err(de::Error::duplicate_field("ref_transactions"));
                            }
                            let parsed: Vec<Result<u64, ParseIntError>> =
                                    map.next_value::<Vec<String>>()?.iter()
                                    .map(|val| { u64::from_str_radix(&val, 16) }).collect();
                            let mut val: Vec<u64> = Vec::with_capacity(parsed.len());
                            for item in parsed.into_iter() {
                                match item {
                                    Err(_) => return Err(de::Error::invalid_value(
                                        Unexpected::Str(&"ref_transactions"), &"valid hex string")),
                                    Ok(valid_item) => val.push(valid_item)
                                }
                            }
                            ref_transactions = Some(val);
                        },
                        Field::Contract => {
                            if contract.is_some() {
                                return Err(de::Error::duplicate_field("contract"));
                            }
                            contract = Some(u64::from_str_radix(
                                &map.next_value::<String>()?, 16)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"contract"), &"valid hex string")})?);
                        },
                        Field::Timestamp => {
                            if timestamp.is_some() {
                                return Err(de::Error::duplicate_field("timestamp"));
                            }
                            timestamp = Some(u64::from_str_radix(
                                &map.next_value::<String>()?, 16)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"timestamp"), &"valid hex string")})?);
                        },
                        Field::Nonce => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(u32::from_str_radix(
                                &map.next_value::<String>()?, 16)
                                .map_err(|_| {de::Error::invalid_value(
                                    Unexpected::Str(&"nonce"), &"valid hex string")})?);
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

                let branch_transaction = branch_transaction.ok_or_else(|| de::Error::missing_field("branch_transaction"))?;
                let trunk_transaction = trunk_transaction.ok_or_else(|| de::Error::missing_field("trunk_transaction"))?;
                let ref_transactions = ref_transactions.ok_or_else(|| de::Error::missing_field("ref_transactions"))?;
                let contract = contract.ok_or_else(|| de::Error::missing_field("contract"))?;
                let timestamp = timestamp.ok_or_else(|| de::Error::missing_field("timestamp"))?;
                let nonce = nonce.ok_or_else(|| de::Error::missing_field("nonce"))?;
                let address = address.ok_or_else(|| de::Error::missing_field("address"))?;
                let signature = signature.ok_or_else(|| de::Error::missing_field("signature"))?;
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;

                Ok(HexEncodedTransaction{
                    branch_transaction: branch_transaction,
                    trunk_transaction: trunk_transaction,
                    ref_transactions: ref_transactions,
                    contract: contract,
                    timestamp: timestamp,
                    nonce: nonce,
                    address: address,
                    signature: signature,
                    data: data
                })
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
        deserializer.deserialize_struct("HexEncodedTransaction", FIELDS, TransactionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use security::ring::digest::SHA512_256;
    use security::keys::PrivateKey;

    #[test]
    fn test_convert() {
        let transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let hex: HexEncodedTransaction = transaction.clone().into();
        let converted: Transaction = hex.into();
        assert_eq!(transaction, converted);
        assert_eq!(transaction.get_branch_hash(), converted.get_branch_hash());
        assert_eq!(transaction.get_trunk_hash(), converted.get_trunk_hash());
        assert_eq!(transaction.get_ref_hashes(), converted.get_ref_hashes());
        assert_eq!(transaction.get_contract(), converted.get_contract());
        assert_eq!(transaction.get_nonce(), converted.get_nonce());
        assert_eq!(transaction.get_address(), converted.get_address());
        assert_eq!(transaction.get_signature(), converted.get_signature());
        assert_eq!(transaction.get_data(), converted.get_data());
    }

    #[test]
    fn test_serialize() {
        let transaction: HexEncodedTransaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis).into();
        let json_value = json!({
            "branch_transaction": "0000000000000000",
            "trunk_transaction": "0000000000000001",
            "ref_transactions": vec!["0000000000000002"],
            "contract": "0000000000000003",
            "timestamp": "0000000000000004",
            "nonce": "00000005",
            "address": "",
            "signature": base64::encode_config(&vec![0; 8192], base64::URL_SAFE),
            "data": TransactionData::Genesis
        });
        assert_eq!(json_value, serde_json::to_value(transaction).unwrap());
    }

    #[test]
    fn test_deserialize() {
        let transaction: HexEncodedTransaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis).into();
        let json_value = json!({
            "branch_transaction": "0000000000000000",
            "trunk_transaction": "0000000000000001",
            "ref_transactions": vec!["0000000000000002"],
            "contract": "0000000000000003",
            "timestamp": "0000000000000004",
            "nonce": "00000005",
            "address": "",
            "signature": base64::encode_config(&vec![0; 8192], base64::URL_SAFE),
            "data": TransactionData::Genesis
        });
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());
    }

    #[test]
    fn test_serialize_deserialize() {
        // Check the transaction is identical after serializing and deserializing
        let transaction: HexEncodedTransaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis).into();
        let json_value = serde_json::to_value(transaction.clone()).unwrap();
        assert_eq!(transaction, serde_json::from_value(json_value).unwrap());

        // Check a signed transaction is identical after serializing and deserializing
        let mut signed_transaction = Transaction::new(0, 1, vec![2], 3, 4, 5, TransactionData::Genesis);
        let mut key = PrivateKey::new(&SHA512_256);
        signed_transaction.sign(&mut key);
        let hex_signed_transaction: HexEncodedTransaction = signed_transaction.into();
        let signed_json_value = serde_json::to_value(hex_signed_transaction.clone()).unwrap();
        assert_eq!(hex_signed_transaction, serde_json::from_value(signed_json_value).unwrap());
    }
}