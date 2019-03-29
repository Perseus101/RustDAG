use std::fmt;
use std::io::Write;

use flate2::write::{GzDecoder, GzEncoder};
use flate2::Compression;

use wasmi::{Error as WasmError, Module};

use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor},
    ser::{self, Serialize, SerializeStruct, Serializer},
};

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct ContractSource {
    code: Vec<u8>,
}

impl ContractSource {
    /// Create contract from raw wasm source
    pub fn new(code: &[u8]) -> Self {
        ContractSource {
            code: code.to_vec(),
        }
    }

    /// Create a wasm module from the contract source
    pub fn get_wasm_module(&self) -> Result<Module, WasmError> {
        Module::from_buffer(&self.code)
    }
}

impl Serialize for ContractSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ContractSource", 1)?;
        // Compress and serialize code
        let mut e = GzEncoder::new(Vec::new(), Compression::best());
        e.write_all(&self.code)
            .map_err(|_| ser::Error::custom("Failed to compress code"))?;
        let bytes = e
            .finish()
            .map_err(|_| ser::Error::custom("Failed to compress code"))?;
        state.serialize_field("code", &base64::encode_config(&bytes, base64::URL_SAFE))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ContractSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Code,
        }

        struct ContractSourceVisitor;

        impl<'de> Visitor<'de> for ContractSourceVisitor {
            type Value = ContractSource;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ContractSource")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ContractSource, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let bytes = base64::decode_config(
                    &seq.next_element::<String>()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                    base64::URL_SAFE,
                )
                .map_err(|_| {
                    de::Error::invalid_value(Unexpected::Str(&"code"), &"valid base64 string")
                })?;

                let mut decoder = GzDecoder::new(Vec::new());
                decoder
                    .write_all(&bytes[..])
                    .map_err(|_| de::Error::custom("Failed to decompress code"))?;
                let code = decoder
                    .finish()
                    .map_err(|_| de::Error::custom("Failed to decompress code"))?;

                Ok(ContractSource::new(&code))
            }

            fn visit_map<V>(self, mut map: V) -> Result<ContractSource, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut code = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Code => {
                            if code.is_some() {
                                return Err(de::Error::duplicate_field("code"));
                            }
                            let bytes = base64::decode_config(
                                &map.next_value::<String>()?,
                                base64::URL_SAFE,
                            )
                            .map_err(|_| {
                                de::Error::invalid_value(
                                    Unexpected::Str(&"code"),
                                    &"valid base64 string",
                                )
                            })?;

                            let mut decoder = GzDecoder::new(Vec::new());
                            decoder
                                .write_all(&bytes[..])
                                .map_err(|_| de::Error::custom("Failed to decompress code"))?;
                            code = Some(
                                decoder
                                    .finish()
                                    .map_err(|_| de::Error::custom("Failed to decompress code"))?,
                            );
                        }
                    }
                }

                let code = code.ok_or_else(|| de::Error::missing_field("code"))?;

                Ok(ContractSource::new(&code))
            }
        }

        const FIELDS: &[&str] = &["code"];
        deserializer.deserialize_struct("ContractSource", FIELDS, ContractSourceVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compress(bytes: &[u8]) -> Vec<u8> {
        let mut e = GzEncoder::new(Vec::new(), Compression::best());
        e.write_all(bytes).expect("Failed to compress bytes");
        e.finish().expect("Failed to compress bytes")
    }

    #[test]
    fn test_contract_source_serialize() {
        let code = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let source = ContractSource::new(&code);
        let json_value = json!({
            "code": base64::encode_config(&compress(&code), base64::URL_SAFE),
        });

        assert_eq!(json_value, serde_json::to_value(source).unwrap());
    }

    #[test]
    fn test_contract_source_deserialize() {
        let code = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let source = ContractSource::new(&code);
        let json_value = json!({
            "code": base64::encode_config(&compress(&code), base64::URL_SAFE),
        });

        assert_eq!(source, serde_json::from_value(json_value).unwrap());
    }

    #[test]
    fn test_contract_source_serialize_deserialize() {
        // Check the transaction is identical after serializing and deserializing
        let code = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let source = ContractSource::new(&code);
        let json_value = serde_json::to_value(source.clone()).unwrap();
        assert_eq!(source, serde_json::from_value(json_value).unwrap());
    }
}
