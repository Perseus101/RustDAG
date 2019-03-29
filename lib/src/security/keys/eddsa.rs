use std::fmt;

use ring::{
    rand,
    signature::{Ed25519KeyPair, Signature, ED25519, verify as ring_verify},
};

use untrusted::Input;

use security::error::KeyError;

use util::array::BigArray;

pub use ring::signature::ED25519_PUBLIC_KEY_LEN;

pub type EdDSAKeyPair = Ed25519KeyPair;

pub type EdDSAPublicKey = [u8; ED25519_PUBLIC_KEY_LEN];

pub const MAX_SIGNATURE_LEN: usize = 1/*tag:SEQUENCE*/ + 2/*len*/ +
    (2 * (1/*tag:INTEGER*/ + 1/*len*/ + 1/*zero*/ + 48/*ec::SCALAR_MAX_BYTES*/));

#[derive(Serialize, Deserialize, Clone)]
pub struct EdSignature {
    #[serde(with = "BigArray")]
    data: [u8; MAX_SIGNATURE_LEN],
    len: usize,
}

impl fmt::Debug for EdSignature {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.data[..self.len].fmt(formatter)
    }
}

impl AsRef<[u8]> for EdSignature {
    fn as_ref(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl From<Signature> for EdSignature {
    fn from(signature: Signature) -> Self {
        let mut buffer = [0u8; MAX_SIGNATURE_LEN];
        let sig_ref = signature.as_ref();
        let len = sig_ref.len();
        (&mut buffer[..len]).copy_from_slice(sig_ref);
        EdSignature { data: buffer , len }
    }
}

pub fn new_key_pair() -> Result<EdDSAKeyPair, KeyError> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair =
        Ed25519KeyPair::from_pkcs8(untrusted::Input::from(pkcs8_bytes.as_ref()))?;
    Ok(key_pair)
}

pub fn get_public_key(key_pair: &EdDSAKeyPair) -> EdDSAPublicKey {
    let mut buffer = [0u8; ED25519_PUBLIC_KEY_LEN];
    buffer[..ED25519_PUBLIC_KEY_LEN]
        .clone_from_slice(key_pair.public_key_bytes());
    buffer
}

pub fn verify(public_key: &EdDSAPublicKey, message: &[u8], signature: &EdSignature) -> bool {
    ring_verify(
        &ED25519,
        Input::from(public_key),
        Input::from(message),
        Input::from(signature.as_ref())
    ).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        const MESSAGE: &[u8] = b"hello, world";

        let key_pair = new_key_pair().unwrap();
        let public_key = get_public_key(&key_pair);
        let signature = key_pair.sign(MESSAGE);

        assert!(verify(&public_key, MESSAGE, &signature.into()));
    }
}