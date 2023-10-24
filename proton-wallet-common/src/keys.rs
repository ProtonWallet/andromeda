
extern crate secp256k1;
extern crate ripemd160;
extern crate sha2;
extern crate rand;
extern crate hex;
extern crate hmac;
extern crate sha3;
extern crate pbkdf2;


use bip39::{Mnemonic, MnemonicType, Language, Seed};
use ripemd160::{Ripemd160, Digest};
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha2::{Sha256, Digest as Sha2Digest};
use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha3::Sha3_256;

pub fn gen_address(phrase: &str, password: &str) -> String {

    let mnemonic = Mnemonic::from_phrase(phrase, Language::English).unwrap();
    // get the phrase
    let parsed: &str = mnemonic.phrase();
    assert_eq!(phrase, parsed);
    println!("phrase: {}", parsed);

    // get the HD wallet seed
    let seed = Seed::new(&mnemonic, password);

    // BIP-32: Use PBKDF2 with HMAC-SHA512 to get 512-bit seed from mnemonic seed
    let mut derived_seed = [0u8; 64];
    pbkdf2::<Hmac<Sha3_256>>(&seed.as_bytes(), b"Bitcoin seed", 2048, &mut derived_seed);

    let secp = Secp256k1::new();
    let private_key = SecretKey::from_slice(&derived_seed[..32]).expect("32 bytes, within curve order");
    
    // Derive the public key
    let public_key = PublicKey::from_secret_key(&secp, &private_key);
    let public_key_serialized = public_key.serialize_uncompressed();
    
    // Generate Bitcoin address (simplified, raw RIPEMD160 hash)
    let sha256 = Sha256::digest(&public_key_serialized);
    let ripemd160 = Ripemd160::digest(&sha256);
    let address = hex::encode(ripemd160);

    println!("Bitcoin Address (simplified): {}", address);

    return address.to_string();
}


pub fn gen_mnemonic() -> String {

    // create a new randomly generated mnemonic phrase
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);

    // get the phrase
    let phrase: &str = mnemonic.phrase();
    println!("phrase: {}", phrase);

    // get the HD wallet seed
    let seed = Seed::new(&mnemonic, "");

    // get the HD wallet seed as raw bytes
    let _seed_bytes: &[u8] = seed.as_bytes();

    // print the HD wallet seed as a hex string
    println!("{:X}", seed);

    return phrase.to_string();
}