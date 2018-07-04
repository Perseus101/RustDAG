use security::hash::sha3::{Sha3_512,Digest};

const MIN_WEIGHT_MAGNITUDE: usize = 2;

pub fn proof_of_work(trunk_nonce: u32, branch_nonce: u32) -> u32 {
    let mut nonce = 0;
    loop {
        if valid_proof(trunk_nonce, branch_nonce, nonce) {
            break;
        }

        nonce += 1;
    }

    nonce
}

pub fn valid_proof(trunk_nonce: u32, branch_nonce: u32, nonce: u32) -> bool {
    let mut guess = String::from(trunk_nonce.to_string());
    guess.push_str(&branch_nonce.to_string());
    guess.push_str(&nonce.to_string());

    let mut hasher = Sha3_512::new();
    hasher.input(guess.as_bytes());
    let hash = hasher.result();

    for b in hash.as_slice()[hash.len()-MIN_WEIGHT_MAGNITUDE..].iter() {
        if *b != 0u8 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_proof() {
        assert!(valid_proof(1, 0, 12645));
        assert!(valid_proof(0, 1, 107752));
    }

    #[test]
    fn test_proof_of_work() {
        assert_eq!(12645, proof_of_work(1, 0));
    }
}
