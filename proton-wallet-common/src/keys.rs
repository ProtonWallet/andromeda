
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    GeneratableKey, GeneratedKey, DerivableKey, ExtendedKey,
};

use bdk::miniscript;
use bdk::bitcoin::{secp256k1::Secp256k1};
use bdk::bitcoin::bip32::DerivationPath;

use bdk::template::Bip84;

use bdk::descriptor;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::miniscript::Tap;
use bdk::Error as BDK_Error;
use std::error::Error;
use std::str::FromStr;


/// Generate a new mnemonic phrase
pub fn gen_mnemonic(count: WordCount) -> String {
    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((count, Language::English)).unwrap();
    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();
    return mnemonic_words;
}


pub fn gen_address(mnemonic_words: &str, passphrase: &str) -> String {

    let secp = Secp256k1::new();

    let mnemonic  = Mnemonic::parse(mnemonic_words).unwrap();
    let mnemonic_with_passphrase = (mnemonic, passphrase);

    // define external and internal derivation key path
    let external_path = DerivationPath::from_str("m/86h/0h/0h/0").unwrap();
    let internal_path = DerivationPath::from_str("m/86h/0h/0h/1").unwrap();


    return "addressOne".to_string();
}


