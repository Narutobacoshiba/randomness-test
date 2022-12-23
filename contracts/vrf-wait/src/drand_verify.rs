mod points;
mod randomness;
mod verify;

use points::{
    g1_from_fixed, g1_from_fixed_unchecked, g1_from_variable, g1_from_variable_unchecked,
    g2_from_fixed, g2_from_fixed_unchecked, g2_from_variable, g2_from_variable_unchecked,
};
use randomness::derive_randomness;
use verify::{verify, verify_step1, verify_step2, VerificationError};
use hex_literal::hex;

/// Public key League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
const PK_LEO_MAINNET: [u8; 48] = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");

pub fn verify_drand_randomness(round: u64, signature: Vec<u8>, previous_signature: Vec<u8>) -> bool{
    let pk = g1_from_fixed(PK_LEO_MAINNET).unwrap();

    match verify(&pk, round, &previous_signature, &signature) {
        Err(err) => return false,
        Ok(valid) => return true 
    }
}

pub fn derive_randomness_from_signature(signature: &[u8]) -> Vec<u8>{
    return derive_randomness(signature).to_vec();
}