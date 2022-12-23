use fff::Field;
use groupy::{CurveAffine, CurveProjective};
use paired::bls12_381::{Bls12, Fq12, G1Affine, G2Affine, G2};
use paired::{Engine, ExpandMsgXmd, HashToCurve, PairingCurveAffine};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

const DOMAIN: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

use super::points::g2_from_variable;

#[derive(Debug)]
pub enum VerificationError {
    InvalidPoint { field: String, msg: String },
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::InvalidPoint { field, msg } => {
                write!(f, "Invalid point for field {}: {}", field, msg)
            }
        }
    }
}

impl Error for VerificationError {}

// Verify checks beacon components to see if they are valid.
pub fn verify(
    pk: &G1Affine,
    round: u64,
    previous_signature: &[u8],
    signature: &[u8],
) -> Result<bool, VerificationError> {
    let msg_on_g2 = verify_step1(round, previous_signature);
    verify_step2(pk, signature, &msg_on_g2)
}

/// First step of the verification.
/// Should not be used directly in most cases. Use [`verify`] instead.
///
/// This API is not stable.
#[doc(hidden)]
pub fn verify_step1(round: u64, previous_signature: &[u8]) -> G2Affine {
    let msg = message(round, previous_signature);
    msg_to_curve(&msg)
}

/// Second step of the verification.
/// Should not be used directly in most cases. Use [`verify`] instead.
///
/// This API is not stable.
#[doc(hidden)]
pub fn verify_step2(
    pk: &G1Affine,
    signature: &[u8],
    msg_on_g2: &G2Affine,
) -> Result<bool, VerificationError> {
    let g1 = G1Affine::one();
    let sigma = match g2_from_variable(signature) {
        Ok(sigma) => sigma,
        Err(err) => {
            return Err(VerificationError::InvalidPoint {
                field: "signature".into(),
                msg: err.to_string(),
            })
        }
    };
    Ok(fast_pairing_equality(&g1, &sigma, pk, msg_on_g2))
}

/// Checks if e(p, q) == e(r, s)
///
/// See https://hackmd.io/@benjaminion/bls12-381#Final-exponentiation.
///
/// Optimized by this trick:
///   Instead of doing e(a,b) (in G2) multiplied by e(-c,d) (in G2)
///   (which is costly is to multiply in G2 because these are very big numbers)
///   we can do FinalExponentiation(MillerLoop( [a,b], [-c,d] )) which is the same
///   in an optimized way.
fn fast_pairing_equality(p: &G1Affine, q: &G2Affine, r: &G1Affine, s: &G2Affine) -> bool {
    let minus_p = {
        let mut out = *p;
        out.negate();
        out
    };
    // "some number of (G1, G2) pairs" are the inputs of the miller loop
    let pair1 = (&minus_p.prepare(), &q.prepare());
    let pair2 = (&r.prepare(), &s.prepare());
    let looped = Bls12::miller_loop([&pair1, &pair2]);
    match Bls12::final_exponentiation(&looped) {
        Some(value) => value == Fq12::one(),
        None => false,
    }
}

fn message(current_round: u64, prev_sig: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(prev_sig);
    hasher.update(round_to_bytes(current_round));
    hasher.finalize().to_vec()
}

/// https://github.com/drand/drand-client/blob/master/wasm/chain/verify.go#L28-L33
#[inline]
fn round_to_bytes(round: u64) -> [u8; 8] {
    round.to_be_bytes()
}

fn msg_to_curve(msg: &[u8]) -> G2Affine {
    let g = <G2 as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(msg, DOMAIN);
    g.into_affine()
}
