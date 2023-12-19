use bdk::bitcoin::secp256k1::rand;
use bdk::bitcoin::secp256k1::rand::Rng;
use bdk::keys::bip39::Language;
use bdk::keys::bip39::WordCount;
use bdk::keys::{GeneratableKey, GeneratedKey};
use bdk::miniscript::BareCtx;

use std::str::FromStr;

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

pub use bdk::keys::bip39::{Language as BdkLanguage, Mnemonic as BdkMnemonic};

use crate::error::Error;

/// Mnemonic phrases are a human-readable version of the private keys.
/// Supported number of words are 12, 15, 18, 21 and 24.
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Mnemonic {
    inner: BdkMnemonic,
}

impl Mnemonic {
    /// Generates Mnemonic with a random entropy
    pub fn new(word_count: WordCount) -> Self {
        // TODO 4: I DON'T KNOW IF THIS IS A DECENT WAY TO GENERATE ENTROPY PLEASE CONFIRM
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 32];
        rng.fill(&mut entropy);

        let generated_key: GeneratedKey<_, BareCtx> =
            BdkMnemonic::generate_with_entropy((word_count, Language::English), entropy).unwrap();
        let mnemonic = BdkMnemonic::parse_in(Language::English, generated_key.to_string()).unwrap();
        Mnemonic { inner: mnemonic }
    }

    /// Parse a Mnemonic with given string
    pub fn from_string(mnemonic: String) -> Result<Self, Error> {
        BdkMnemonic::from_str(&mnemonic)
            .map(|m| Mnemonic { inner: m })
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }

     /// Returns Mnemonic as string
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }

    pub fn to_words(&self) -> Vec<String> {
        self.inner.word_iter().map(|word| String::from(word)).collect()
    }

    pub fn inner(&self) -> BdkMnemonic {
        self.inner.clone()
    }
}
