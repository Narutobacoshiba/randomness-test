//use rsa::{BigUint, PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};
use sha2::{Sha512, Digest};
use std::{fmt::Write, num::ParseIntError};
use openssl::rsa::{Rsa, Padding};
use openssl::sign::Verifier;
use openssl::bn::BigNum;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

fn u32_to_arr_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

fn u64_to_arr_u8(x:u64) -> [u8;8] {
    let b1 : u8 = ((x >> 56) & 0xffff) as u8;
    let b2 : u8 = ((x >> 48) & 0xffff) as u8;
    let b3 : u8 = ((x >> 40) & 0xffff) as u8;
    let b4 : u8 = ((x >> 32) & 0xffff) as u8;
    let b5 : u8 = ((x >> 24) & 0xffff) as u8;
    let b6 : u8 = ((x >> 16) & 0xffff) as u8;
    let b7 : u8 = ((x >> 8) & 0xffff) as u8;
    let b8 : u8 = (x & 0xffff) as u8;
    return [b1, b2, b3, b4, b5, b6, b7, b8];
}

fn hash_serialize(hex_s: &str) -> Vec<u8>{
    let mut s = decode_hex(hex_s).unwrap();
    let size = s.len() as u64;
    let mut z = u64_to_arr_u8(size).to_vec();
    z.append(&mut s);
    return z;
}

fn str_serialize(s: &str) -> Vec<u8>{
    let mut s = s.as_bytes().to_vec();
    let size = s.len() as u64;
    let mut z = u64_to_arr_u8(size).to_vec();
    z.append(&mut s);
    return z;
}

fn hash_json(){
    let mut sig = base64::decode("0K510lwXPxj8AHPV+cQoYuW4snOtjd8NTytz16XC8PHSOMXJNOW3yVynSiuVf20mc1fLHbmKjP08//TfqPyIYWd40A9OA+iJcHz+VXRgwCzSH/RK2nnxqN7uuah2xCXXerfcW5g/sRkRHrPZIjoTPVR/adXdjZBQ6q4Wb0JXItYpFv5aUCEBQWa2izq7Ax+ZNZI0PjifI5zQacPheVxoyEGYB2TtsWWYIHDI+M5afK0E0yyOjiR+emozmD3M3KgLpYq8UkaGR4rSNNgNsrLTyupDebOouRlyevXmKZURWmXZnJlW8sJKrvvPGnUQrRSDbpxBOuaBpg0SPozIr8Avv1CJCngcaDumjQFCuesQvTjQACBwrsqGZoSSHtw3QgWQdcfPnZBWOQ3jlaVk897fCEI6TYOnT+U9spvFmVdtmSVhaeftmQ5+yDoYhe5YHf2AQcmUxlikyhBmob4Fv6VDmKgpy5Ke30zlaNhFdXonvQZk+wlqlsaYk7cnmtaxrMcHlQcpVHZRRLNc5FHg0nFepe0z/T30XmFEyyOQlrAmpwZ6tKwksXDykQW5AyPUY6+esCl3rDXdt3GFis8D6/WldOKuMiGKW/JN7w9zR8W7NGxJ4INv3eO7Er8yJoxyMvD6eQ3STO3pAjBZ37e43mx7F/pnxaFOPPFrk9dMcWdCPmw=").unwrap();

    let data = base64::decode("eyJtZXRob2QiOiJnZW5lcmF0ZVNpZ25lZEludGVnZXJzIiwiaGFzaGVkQXBpS2V5IjoiSUVicnY4NzFLZnBsdjNmdkFaWG9rRFg2S1o1N0pES2wyajhLNktRZkxnRk12MDF6ZktsWnJweXFnQ3MyRE9rRVg4LzcvQ2xIZm0yZHFveFh3VVJkTHc9PSIsIm4iOjMyLCJtaW4iOjAsIm1heCI6MjU1LCJyZXBsYWNlbWVudCI6dHJ1ZSwiYmFzZSI6MTAsInByZWdlbmVyYXRlZFJhbmRvbWl6YXRpb24iOm51bGwsImRhdGEiOlszNCwxNTIsMTIyLDEyLDExOCw1MywxOTAsMzcsMjQsMTAsMCwxMDEsNjAsMTQ0LDE4NywxMSwxNzcsMTE0LDM3LDIxNywxNDksMjI0LDI2LDg3LDI5LDE0OSwxNTAsNzQsMTM2LDQ1LDE5OSwyMzJdLCJsaWNlbnNlIjp7InR5cGUiOiJkZXZlbG9wZXIiLCJ0ZXh0IjoiUmFuZG9tIHZhbHVlcyBsaWNlbnNlZCBzdHJpY3RseSBmb3IgZGV2ZWxvcG1lbnQgYW5kIHRlc3Rpbmcgb25seSIsImluZm9VcmwiOm51bGx9LCJsaWNlbnNlRGF0YSI6bnVsbCwidXNlckRhdGEiOm51bGwsInRpY2tldERhdGEiOm51bGwsImNvbXBsZXRpb25UaW1lIjoiMjAyMi0xMi0wOCAwMjo1MjoxNVoiLCJzZXJpYWxOdW1iZXIiOjl9").unwrap();
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(&data);
    // read hash digest and consume hasher
    let mut result = hasher.finalize();
    println!("{}",encode_hex(&result));
    println!("{}",encode_hex(&sig));

    let n: BigNum = BigNum::from_hex_str("ecedc74162e74f30828ffab0a08e2f8ff4fddb7ef07bbe2bc1c256db0e12bb320a565027e7285a25c69e429769987c2642ddda53c1b56daee7df197b85d78f921f9a12460cde254e84965d9022a3cf0db1ee55124089d992c827b3c47888692524f2275fa7e606312bb7562b8c8f01e47ab3de4a226e4a8866056e67541f26881b9acad3eb88a68220dd786dd70dc398e320f34bbdf86cda9150d6216b76839f0bf1aee6f23217d6b41976cba9d72836de30a27d356bbbdb757b2fe04615e12f60c3eaf22791549ef271abca7925c4a22f46be0cc28eecb618124e5ece353b97f4ed59ea1b1722eaeab26e5120af44a83444d816726c49592bcb24cfb4eee58798dd160e1098705411fcdf71640c9318f82db0ef447327e5422ba1f900ee0fbded67ff2109d9ce195987e0e021bde38d70f9d06a89b1dedc774a23259bb319fe812d267c836299389dcab41d6efe76781d541474fe99368a77984c7b3226abef04838d1cc68386b27f11daf293ad13aa3ca5ed1dee556edd74c70bd90be6a6775ea95de92c7db49d99436a038d33e53c885818c2dd78485799852b8670c2869389ad6bec6ff7a1e0cdfcb1651c70141397db01bd6464adb4826b3971640f98e4a38f109dcd211f068ca14dc1b77c064f589372e76e8712a7713cd81543d608b8cd177d32d0610a519cfffc62f12e56ac5868f25fac67e742abf8ae5582d39065").unwrap();
    let e: BigNum = BigNum::from_hex_str("010001").unwrap();

    let rsa = Rsa::from_public_components(n,e).expect("error");
    let keypair = PKey::from_rsa(rsa).unwrap();

    //let padding = PaddingScheme::new_pkcs1v15_sign();
    //let verify = public_key.verify(padding, &result, &sig);
    //let mut buff = vec![0; rsa.size() as usize];

    //let encrypted_len = rsa.public_encrypt(&sig, &mut buff, Padding::PKCS1).unwrap();
    //println!("{}",encode_hex(&buff));

    let mut verifier = Verifier::new(MessageDigest::sha512(), &keypair).unwrap();
    verifier.update(&data).unwrap();
    let verify = verifier.verify(&sig).unwrap();
    
    if verify {
        println!("success");
    }else{
        println!("error");
    }
}

fn main() {
    // height 8737
    let mut rng = rand::thread_rng();
    /*
    let bits = 4096;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    
    // Encrypt
    let data = b"hello world";

    let mut hasher = Sha512::new();
    // write input message
    hasher.update(data);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    println!("{:x?}",&result);
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let signature = private_key.sign(padding, &result).expect("error");

    // Decrypt
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let verify = public_key.verify(padding, &result, &signature);
    match verify {
        Ok(()) => println!("ok"),
        Err(_) => println!("err")
    }
    */

    /*
    const p: &str = "CAECDD5F31509ACE3E568C50236958A9771CEEF87B3D4027BC9C37ED7A11AB026487610328C9027392E119FDDCB356D884DBD9FE2CA467B87B2B67E61E1C477A";
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(decode_hex(&p).unwrap());
    // read hash digest and consume hasher
    let result = hasher.finalize();
    println!("{:x?}",result)*/
    /*
    let pem: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAtsQsUV8QpqrygsY+2+JCQ6Fw8/omM71IM2N/R8pPbzbgOl0p78MZ
GsgPOQ2HSznjD0FPzsH8oO2B5Uftws04LHb2HJAYlz25+lN5cqfHAfa3fgmC38Ff
wBkn7l582UtPWZ/wcBOnyCgb3yLcvJrXyrt8QxHJgvWO23ITrUVYszImbXQ67YGS
0YhMrbixRzmo2tpm3JcIBtnHrEUMsT0NfFdfsZhTT8YbxBvA8FdODgEwx7u/vf3J
9qbi4+Kv8cvqyJuleIRSjVXPsIMnoejIn04APPKIjpMyQdnWlby7rNyQtE4+CV+j
cFjqJbE/Xilcvqxt6DirjFCvYeKYl1uHLwIDAQAB
-----END RSA PUBLIC KEY-----";
    */
    
    let mut f1 = str_serialize("https://beacon.nist.gov/beacon/2.0/chain/2/pulse/8737");
    let mut f2 = str_serialize("2.0");
    let mut f3:Vec<u8> = u32_to_arr_u8(0).to_vec();
    let mut f4:Vec<u8> = u32_to_arr_u8(60000).to_vec();
    let mut f5 = hash_serialize("ed8e1b9745e337e38745a54579d97b21562cb703e44926666a1d6499a9e3399448cdf6b1403ec056314c129910e82b0e8af48c1e062a88828e178dfc82983c2b");
    let mut f6:Vec<u8> = u64_to_arr_u8(2).to_vec();
    let mut f7:Vec<u8> = u64_to_arr_u8(8737).to_vec(); 
    let mut f8 = str_serialize("2022-09-27T21:09:00.000Z");
    let mut f9 = hash_serialize("1C720EDEED908E96ABBF438FA49B91003E0C96E03E996A59F270B881BC9C164C4E1093848D5519AD806A74DC977DD33F55FCEE989DA0B4051257C58CA190F81B");
    let mut f10 = hash_serialize("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let mut f11 = u64_to_arr_u8(0).to_vec();
    let mut f12 = hash_serialize("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let mut f13 = hash_serialize("BD2A395E487F456467E26308364B1DBBDA7552FB49B5452E388E71BF16C98C167C555386C1E1AD4298933A21976E64385EAEA25C1CC44B3BEB7819E39ED29231");
    let mut f14 = hash_serialize("BABE36FF28AAF1F325283778356CDDB2CF92787CB87DCE7A960F7E5AE7815218F1451E32F283EA5C641E6D03A296EADE771AD72BAAFA4AC9387FE6C1B4572784");
    let mut f15 = hash_serialize("71FB7BD41278DC81126BAC82FEFDF56F6A72D5691AD6833CD079667551976B20EDF50114AA53BE62F19438192CCFF0B9601B5A01C52F384DF3051DA0C1338D40");
    let mut f16 = hash_serialize("7874173C3CA08249238E48837032E4D60E6144A6719E665DF87D997D59423F1609C8A365BF50B758C2872F35235CF7CC6440893AA3E781EF904507F8116FF576");
    let mut f17 = hash_serialize("7874173C3CA08249238E48837032E4D60E6144A6719E665DF87D997D59423F1609C8A365BF50B758C2872F35235CF7CC6440893AA3E781EF904507F8116FF576");
    let mut f18 = hash_serialize("7F89F7684506570DAAB3AC01522E5DD24F2B084490A102401DA8FF44FA3C910D8B8B00BCF329EDAE1825CC34467453260AFA349121B4A8C8A4326EB54F7370D1");
    let mut f19 = u32_to_arr_u8(0).to_vec();
    let mut f20 = hash_serialize("94E45C2FA91EC9D3C2D12A995CF153D797A3E2F48E84107D1797C84EECBF97D2152F7DB179D4428A212B304FAEB03BBF13B78819F85D31B44D1A463D59DB9FBD9C9DF9C3544DA406F3729A39DD08D4A171F9843EF206FF5E8915CECC2D663198E792E95259310D36787705A2C754B996957483F8C488E25F117336101EFEC40C6D7383BD263D83B6EA5C0605386C119ED0574396F35938013C97F615A63144F87DE07DECE80C371CD6899DB5737AFF5166E88FEE6843EF9644DFB68732869F4043064B8D1228C196BDA9A37BCAD9806917CD65EB69E4F2F2EEE60CCE952B273E0E6512434F885C4097B05AABC3443980658BA35C3EB7D4394E421A3BBBA24E9F25F657E68B4E986B8099D9F1036A344E7068C0FE5B82CF1C04AF990D98875BCBC488FDF04A2FB84BCBB5C827C513B4A9F43D709AB89CE6CFCBF7BA46FB53B08A789D0609DCC674F5529A30AC05255087B7FE83DAC18BD4E59D6B9789D0183B9E6D829D82BFA4227B6A494500F6A25440EA7225790EBB1BB2B0C6099FA92E995FB5B64E9D9AFB354C9D1F9B7913671F8E52852F4DFDE8F255FDB468818A033BC16DC3E4C056F502557A00AF0D4BA50BB06EBF45BE2442F270391A9DDA2475E3640A33646642CDCB352B998267C646DBC219C135CA28AC12242B31C6A3DA4EE858A2F7B15DBF9526A5F9898F045758A55C75DD1F7DA7A3D93BF8A1316505A8B954");

    f1.append(&mut f2);
    f1.append(&mut f3);
    f1.append(&mut f4);
    f1.append(&mut f5);
    f1.append(&mut f6);
    f1.append(&mut f7);
    f1.append(&mut f8);
    f1.append(&mut f9);
    f1.append(&mut f10);
    f1.append(&mut f11);
    f1.append(&mut f12);
    f1.append(&mut f13);
    f1.append(&mut f14);
    f1.append(&mut f15);
    f1.append(&mut f16);
    f1.append(&mut f17);
    f1.append(&mut f18);
    f1.append(&mut f19);
    f1.append(&mut f20);

    let sig = decode_hex("94E45C2FA91EC9D3C2D12A995CF153D797A3E2F48E84107D1797C84EECBF97D2152F7DB179D4428A212B304FAEB03BBF13B78819F85D31B44D1A463D59DB9FBD9C9DF9C3544DA406F3729A39DD08D4A171F9843EF206FF5E8915CECC2D663198E792E95259310D36787705A2C754B996957483F8C488E25F117336101EFEC40C6D7383BD263D83B6EA5C0605386C119ED0574396F35938013C97F615A63144F87DE07DECE80C371CD6899DB5737AFF5166E88FEE6843EF9644DFB68732869F4043064B8D1228C196BDA9A37BCAD9806917CD65EB69E4F2F2EEE60CCE952B273E0E6512434F885C4097B05AABC3443980658BA35C3EB7D4394E421A3BBBA24E9F25F657E68B4E986B8099D9F1036A344E7068C0FE5B82CF1C04AF990D98875BCBC488FDF04A2FB84BCBB5C827C513B4A9F43D709AB89CE6CFCBF7BA46FB53B08A789D0609DCC674F5529A30AC05255087B7FE83DAC18BD4E59D6B9789D0183B9E6D829D82BFA4227B6A494500F6A25440EA7225790EBB1BB2B0C6099FA92E995FB5B64E9D9AFB354C9D1F9B7913671F8E52852F4DFDE8F255FDB468818A033BC16DC3E4C056F502557A00AF0D4BA50BB06EBF45BE2442F270391A9DDA2475E3640A33646642CDCB352B998267C646DBC219C135CA28AC12242B31C6A3DA4EE858A2F7B15DBF9526A5F9898F045758A55C75DD1F7DA7A3D93BF8A1316505A8B954").unwrap();
    
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(&f1);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    println!("{}\n",encode_hex(&result));

    let n: BigNum = BigNum::from_hex_str("c3fa69b08a0ef706c91fe990c09e980feda7ccd2d1f8389f664a2281e5cb13a80dd64855a451cc4ce10fff91192d8cec406f58c9735193414853f26ee4e0ab93858fb808a7c337a753d626f2b49054d491bc20ed0e2b74adc60525c09a11f9f64ea67e89f76a2cc9421fdd81e2929496cfb7c5c0e837da34459c7b280aee6a6b18eef9d15daa5b53b3c7fb45f2572cd6666a6b86cebaf27d1d62c7d1d96780140a69478589014e9f1df71a95f677ec329dd54f77688a6641be795bb59fc9da4f82bf7e9425fe2fa7d9b74e8314d1e37afffd85e66a8c891b2fc4febb89959f86b486f8e48ccaac7ed07a6ce67d8aa5648b6eb8b8c9c6d9a719476db20a5f152e51947b7f42a0a5b36ea10ff53b4c4266154e2945390fdc103a6f6c8e265e1ea95cf66eedd5b45e68fa0fabfad6d66729e7b813fc066a452966a127567d1b43423e46f923243980809b023f513d4ba09dad1cd8b896975373b1851212237916c59031a5b93433b157c41ec79f23ee07885fdeb04122f35198d83e14bba7c78ef3e1952444add268f5ffd451704fb1632701d5744839d14b0422eb90d4a211019c4f122fc5b7267d71a0a4958d8e7be9730c3cdf0ed01022cf7fffcbc1a8ed337a71fab4554be413b41799ddab700a38549fc9511154ae1841dadb0747afa773a4c26977604bfe05a49523cc64e5689129dda4180fb60d38f333a85333260848a7").unwrap();
    let e: BigNum = BigNum::from_hex_str("010001").unwrap();

    let rsa = Rsa::from_public_components(n,e).expect("error");
    let keypair = PKey::from_rsa(rsa).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha512(), &keypair).unwrap();
    verifier.update(&result).unwrap();
    let verify = verifier.verify(&sig).unwrap();
    println!("{}",verify);
    
}
