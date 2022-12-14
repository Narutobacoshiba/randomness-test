use openssl::rsa::{Rsa, Padding};
use openssl::sign::Verifier;
use openssl::bn::BigNum;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;

const RANDOM_ORG_PUBLIC_KEY_N: &str = "ecedc74162e74f30828ffab0a08e2f8ff4fddb7ef07bbe2bc1c256db0e12bb320a565027e7285a25c69e429769987c2642ddda53c1b56daee7df197b85d78f921f9a12460cde254e84965d9022a3cf0db1ee55124089d992c827b3c47888692524f2275fa7e606312bb7562b8c8f01e47ab3de4a226e4a8866056e67541f26881b9acad3eb88a68220dd786dd70dc398e320f34bbdf86cda9150d6216b76839f0bf1aee6f23217d6b41976cba9d72836de30a27d356bbbdb757b2fe04615e12f60c3eaf22791549ef271abca7925c4a22f46be0cc28eecb618124e5ece353b97f4ed59ea1b1722eaeab26e5120af44a83444d816726c49592bcb24cfb4eee58798dd160e1098705411fcdf71640c9318f82db0ef447327e5422ba1f900ee0fbded67ff2109d9ce195987e0e021bde38d70f9d06a89b1dedc774a23259bb319fe812d267c836299389dcab41d6efe76781d541474fe99368a77984c7b3226abef04838d1cc68386b27f11daf293ad13aa3ca5ed1dee556edd74c70bd90be6a6775ea95de92c7db49d99436a038d33e53c885818c2dd78485799852b8670c2869389ad6bec6ff7a1e0cdfcb1651c70141397db01bd6464adb4826b3971640f98e4a38f109dcd211f068ca14dc1b77c064f589372e76e8712a7713cd81543d608b8cd177d32d0610a519cfffc62f12e56ac5868f25fac67e742abf8ae5582d39065";
const RANDOM_ORG_PUBLIC_KEY_E: &str = "010001";

pub fn VerifyRandomOrgSig(data: Vec<u8>, signature: Vec<u8>) -> bool{
    let n: BigNum = BigNum::from_hex_str(RANDOM_ORG_PUBLIC_KEY_N).unwrap();    
    let e: BigNum = BigNum::from_hex_str(RANDOM_ORG_PUBLIC_KEY_E).unwrap();

    // Create Rsa public key from n,e components
    let rsa = Rsa::from_public_components(n,e).expect("error");
    let keypair = PKey::from_rsa(rsa).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha512(), &keypair).unwrap();
    verifier.update(&data).unwrap();
    let verify = verifier.verify(&signature).unwrap();
    drop(verifier);

    return verify;
}