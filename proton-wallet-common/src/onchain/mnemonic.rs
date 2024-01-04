use bdk::bitcoin::secp256k1::rand;
use bdk::bitcoin::secp256k1::rand::Rng;
use bdk::keys::bip39::Language;
use bdk::keys::bip39::WordCount;
use bdk::keys::{GeneratableKey, GeneratedKey};
use bdk::miniscript::BareCtx;

use std::str::FromStr;

pub use bdk::keys::bip39::{Language as BdkLanguage, Mnemonic as BdkMnemonic};

use crate::common::error::Error;

#[derive(Debug)]
pub struct Mnemonic {
    inner: BdkMnemonic,
}

impl Mnemonic {
    /// Generates Mnemonic with a random entropy
    pub fn new(word_count: WordCount) -> Result<Self, Error> {
        // TODO 4: I DON'T KNOW IF THIS IS A DECENT WAY TO GENERATE ENTROPY PLEASE CONFIRM
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 32];
        rng.fill(&mut entropy);

        let generated_key: GeneratedKey<_, BareCtx> =
            BdkMnemonic::generate_with_entropy((word_count, Language::English), entropy).map_err(|e| match e {
                Some(e) => e.into(),
                _ => Error::Bip39 { error: None },
            })?;

        let mnemonic = BdkMnemonic::parse_in(Language::English, generated_key.to_string()).map_err(|e| e.into())?;

        Ok(Mnemonic { inner: mnemonic })
    }

    /// Parse a Mnemonic with given string
    pub fn from_string(mnemonic: String) -> Result<Self, Error> {
        BdkMnemonic::from_str(&mnemonic)
            .map(|m| Mnemonic { inner: m })
            .map_err(|_| Error::InvalidMnemonic)
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

#[cfg(test)]
mod tests {
    use bdk::keys::bip39::Language;

    use crate::common::error::Error;

    use super::Mnemonic;

    #[test]
    fn should_create_mnemonic_from_string() {
        let mnemonic = Mnemonic::from_string(
            "affair recycle please start moment film grain myself flight issue artwork silver".to_string(),
        )
        .unwrap();

        assert_eq!(mnemonic.inner.language(), Language::English);
        assert_eq!(mnemonic.inner.word_count(), 12);
        assert_eq!(
            mnemonic.inner.to_string(),
            "affair recycle please start moment film grain myself flight issue artwork silver".to_string()
        );

        assert_eq!(
            mnemonic.inner.to_seed("").len(),
            "adfbd2848041a85dfaed1feddb38e3e08680ff823a6d4e04708199ea3245b31a"
                .to_uppercase()
                .as_bytes()
                .len()
        );

        // with passphrase
        assert_eq!(
            mnemonic.inner.to_seed("this_is_a_test_passphrase").len(),
            "ae8982ff363e0fe7721ce9c6b98bc46d70ebdf37dd53d6de454e33bb41113a93"
                .to_uppercase()
                .as_bytes()
                .len()
        );
    }

    #[test]
    fn should_throw_when_mispelled_word() {
        // affair is mispelled
        let mnemonic_error = Mnemonic::from_string(
            "afair recycle please start moment film grain myself flight issue artwork silver".to_string(),
        )
        .err()
        .unwrap();

        assert!(match mnemonic_error {
            Error::InvalidMnemonic => true,
            _ => false,
        });
    }

    #[test]
    fn should_throw_when_invalid_word() {
        // ogre is not a valid word
        let mnemonic_error = Mnemonic::from_string(
            "ogre recycle please start moment film grain myself flight issue artwork silver".to_string(),
        )
        .err()
        .unwrap();

        assert!(match mnemonic_error {
            Error::InvalidMnemonic => true,
            _ => false,
        });
    }

    #[test]
    fn should_throw_when_invalid_word_count() {
        // there are only 11 words
        let mnemonic_error = Mnemonic::from_string(
            "recycle please start moment film grain myself flight issue artwork silver".to_string(),
        )
        .err()
        .unwrap();

        assert!(match mnemonic_error {
            Error::InvalidMnemonic => true,
            _ => false,
        });
    }

    #[test]
    fn should_throw_when_invalid_lang() {
        // lang is in French
        let mnemonic_error = Mnemonic::from_string(
            "entourer étroit digne déposer plateau finir magasin élargir berline sergent fugitif bistouri".to_string(),
        )
        .err()
        .unwrap();

        assert!(match mnemonic_error {
            Error::InvalidMnemonic => true,
            _ => false,
        });
    }

    #[test]
    fn should_return_word_vector() {
        // lang is in French
        let mnemonic = Mnemonic::from_string(
            "affair recycle please start moment film grain myself flight issue artwork silver".to_string(),
        )
        .unwrap();

        assert_eq!(
            mnemonic.to_words(),
            vec![
                "affair", "recycle", "please", "start", "moment", "film", "grain", "myself", "flight", "issue",
                "artwork", "silver"
            ]
        );
    }
}
