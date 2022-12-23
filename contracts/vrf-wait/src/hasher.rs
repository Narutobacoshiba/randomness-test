use sha2::{Sha256, Sha512, Digest};


pub fn sha256_hash(string: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(string);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result.to_vec();
}

pub fn sha512_hash(string: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(string);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result.to_vec();
}