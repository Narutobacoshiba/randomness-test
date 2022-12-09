use sha2::{Sha512, Digest};


pub fn Sha512Hash(string: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(&string);
            // read hash digest and consume hasher
    let result = hasher.finalize();

    return result.to_vec();
}