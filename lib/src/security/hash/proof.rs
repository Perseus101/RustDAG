use security::hash::sha3::{Sha3_512,Digest};

const MIN_WEIGHT_MAGNITUDE: usize = 2;

pub fn proof_of_work(trunk_nonce: u32, branch_nonce: u32) -> u32 {
    (0u32..)
        .find(|nonce| valid_proof(trunk_nonce, branch_nonce, *nonce))
        .expect("No valid proof of work was found")
}

pub fn valid_proof(trunk_nonce: u32, branch_nonce: u32, nonce: u32) -> bool {
    let guess = nonces_to_bytes(trunk_nonce, branch_nonce, nonce);

    let mut hasher = Sha3_512::new();
    hasher.input(&guess);
    let hash = hasher.result();

    for b in hash.as_slice()[hash.len()-MIN_WEIGHT_MAGNITUDE..].iter() {
        if *b != 0u8 {
            return false;
        }
    }
    true
}

fn nonces_to_bytes(trunk_nonce: u32, branch_nonce: u32, nonce: u32) -> [u8;12] {
    let mut nonces: u128 =
          (u128::from(trunk_nonce.to_le()) << 64)
        + (u128::from(branch_nonce.to_le()) << 32)
        + u128::from(nonce.to_le());
        //to_le converts to little endian

    let mut bytes = [0u8; 12];
    for i in (0..12).rev() {
        bytes[i] = (nonces & 0xff) as u8;
        nonces >>= 8;
    }

    bytes
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;

    #[test]
    fn test_nonces_to_bytes() {
        assert_eq!(nonces_to_bytes(42, 12, 0x04030201), [0, 0, 0, 42, 0, 0, 0, 12, 4, 3, 2, 1]);
    }

    #[test]
    fn test_valid_proof() {
        assert!(valid_proof(1, 0, 136516));
        assert!(valid_proof(0, 1, 29972));
    }

    #[bench]
    #[ignore]
    fn bench_proof_of_work(b: &mut test::Bencher) {
        b.iter(|| assert_eq!(136516, proof_of_work(1, 0)));
    }

    #[bench]
    fn bench_valid_proof(b: &mut test::Bencher) {
        b.iter(|| valid_proof(25565, 12345, 98765));
    }
}
