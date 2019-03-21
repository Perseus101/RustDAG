use std::hash::Hasher;

use std::mem::transmute;

use security::hash::sha3::{Digest, Sha3_512};

pub struct Sha3Hasher {
    hasher: Sha3_512,
}

impl Default for Sha3Hasher {
    fn default() -> Self {
        Sha3Hasher::new()
    }
}

impl Hasher for Sha3Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.input(bytes);
    }

    fn finish(&self) -> u64 {
        let result = self.hasher.clone().result();
        _bytes_to_u64(result.as_slice())
    }
}

impl Sha3Hasher {
    pub fn new() -> Sha3Hasher {
        Sha3Hasher {
            hasher: Sha3_512::new(),
        }
    }

    pub fn finish_bytes(&self) -> Vec<u8> {
        self.hasher.clone().result().to_vec()
    }
}

fn _bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut buffer = [0u8; 8];
    buffer[0] = bytes[7];
    buffer[1] = bytes[6];
    buffer[2] = bytes[5];
    buffer[3] = bytes[4];
    buffer[4] = bytes[3];
    buffer[5] = bytes[2];
    buffer[6] = bytes[1];
    buffer[7] = bytes[0];
    unsafe { transmute::<[u8; 8], u64>(buffer) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bytes_to_u64() {
        assert_eq!(0, _bytes_to_u64(&[0, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(0x01, _bytes_to_u64(&[0, 0, 0, 0, 0, 0, 0, 0x01]));
        assert_eq!(
            0x0101010101010101,
            _bytes_to_u64(&[0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])
        );
        assert_eq!(
            0xffffffffffffffff,
            _bytes_to_u64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        );
    }
}
