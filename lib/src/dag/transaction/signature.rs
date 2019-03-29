use std::hash::{Hash, Hasher};

use security::keys::eddsa::{EdDSAPublicKey, EdSignature};
use security::hash::hasher::Sha3Hasher;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransactionSignature {
    Unsigned,
    EdDSA {
        public_key: EdDSAPublicKey,
        signature: EdSignature
    }
}

impl Default for TransactionSignature {
    fn default() -> Self {
        TransactionSignature::Unsigned
    }
}

impl PartialEq<TransactionSignature> for TransactionSignature {
    fn eq(&self, other: &TransactionSignature) -> bool {
        match self {
            TransactionSignature::Unsigned => {
                if let TransactionSignature::Unsigned = other { true }
                else { false }
            }
            TransactionSignature::EdDSA { .. } => {
                if let TransactionSignature::EdDSA { .. } = other { true }
                else { false }
            }
        }
    }
}


impl TransactionSignature {
    pub fn get_address(&self) -> u64 {
        match self {
            TransactionSignature::Unsigned => 0,
            TransactionSignature::EdDSA { public_key, .. } => {
                let mut hasher = Sha3Hasher::new();
                public_key.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
}
