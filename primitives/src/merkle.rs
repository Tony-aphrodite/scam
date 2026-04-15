use alloy_primitives::{B256, U256};
use sha2::{Digest, Sha256};

pub fn calculate_merkle_root(mut hashes: Vec<B256>) -> B256 {
    if hashes.is_empty() {
        return B256::from(U256::from(0));
    }

    while hashes.len() > 1 {
        let mut next = Vec::new();
        for pair in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            if pair.len() == 2 {
                hasher.update(&pair[0]);
                hasher.update(&pair[1]);
            } else {
                hasher.update(&pair[0]);
                hasher.update(&pair[0]);
            }
            next.push(B256::from_slice(&hasher.finalize()));
        }
        hashes = next;
    }
    
    hashes[0]
}