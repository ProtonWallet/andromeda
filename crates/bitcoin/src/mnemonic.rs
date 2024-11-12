use std::str::FromStr;

use bdk_wallet::{
    bitcoin::secp256k1::{rand, rand::Rng},
    keys::{
        bip39::{Language, Mnemonic as BdkMnemonic, WordCount},
        GeneratableKey, GeneratedKey,
    },
    miniscript::BareCtx,
};

use crate::error::Error;

#[derive(Debug)]
pub struct Mnemonic {
    inner: BdkMnemonic,
}

/// Returns a vector of words from the English language word list that start
/// with the given prefix.
///
/// # Arguments
///
/// * `word_start` - A string representing the prefix for which autocomplete
///   suggestions are generated.
///
/// # Note
///
/// This function relies on the `word_list` method of the `Language::English`
/// enum to obtain the English language word list. Ensure that the necessary
/// dependencies are available and properly configured.
///
/// # Panics
///
/// This function should not panic under normal circumstances. If panics occur,
/// they may indicate issues with the underlying word list or language enum
/// implementation.
///
/// # Returns
///
/// A vector of strings representing words that start with the provided prefix.
///
/// # Examples
///
/// ```rust
/// use andromeda_bitcoin::mnemonic::get_words_autocomplete;
///
/// let result = get_words_autocomplete(String::from("pre"));
/// assert_eq!(result, vec!["predict", "prefer", "prepare", "present", "pretty", "prevent"]);
/// ```
pub fn get_words_autocomplete(word_start: String) -> Vec<String> {
    Language::English
        .word_list()
        .iter()
        .filter(|word| word.starts_with(&word_start))
        .map(|word| word.to_string())
        .collect::<Vec<_>>()
}

/// Wrapper around BDK's Mnemonic struct
impl Mnemonic {
    /// Generates a new `Mnemonic` with a random entropy.
    ///
    /// # Arguments
    ///
    /// * `word_count` - number of words the mnemonic should be composed with
    ///
    /// # Note
    ///
    /// This function isn't pure since it relies on `rand::thread_rng` to
    /// generate entropy.
    ///
    /// # Returns
    ///
    /// A Mnemonic
    ///
    /// # Examples
    ///
    /// ```rust
    /// use andromeda_bitcoin::mnemonic::Mnemonic;
    /// use bdk_wallet::keys::bip39::WordCount;
    ///
    /// let result = Mnemonic::new(WordCount::Words12);
    /// println!("{:?}", result)
    /// ```
    pub fn new(word_count: WordCount) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 32];
        rng.fill(&mut entropy);

        let generated_key: GeneratedKey<_, BareCtx> =
            BdkMnemonic::generate_with_entropy((word_count, Language::English), entropy).expect("should not fail");

        let mnemonic = BdkMnemonic::parse_in(Language::English, generated_key.to_string())?;

        Ok(Mnemonic { inner: mnemonic })
    }

    pub fn new_with(entropy: &[u8]) -> Result<Self, Error> {
        let mnemonic = BdkMnemonic::from_entropy_in(Language::English, entropy)?;
        Ok(Mnemonic { inner: mnemonic })
    }

    /// Parses a string to a `Mnemonic`.
    ///
    /// # Arguments
    ///
    /// * `mnemonic` - the mnemonic string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use andromeda_bitcoin::mnemonic::Mnemonic;
    ///
    /// let result = Mnemonic::from_string("desk prevent enhance husband hungry idle member vessel room moment simple behave".to_string());
    /// println!("{:?}", result)
    /// ```
    pub fn from_string(mnemonic: String) -> Result<Self, Error> {
        let mnemonic = BdkMnemonic::from_str(&mnemonic).map(|m| Mnemonic { inner: m })?;

        Ok(mnemonic)
    }

    /// serialize a `Mnemonic` to a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use andromeda_bitcoin::mnemonic::Mnemonic;
    ///
    /// let str_mnemonic = "desk prevent enhance husband hungry idle member vessel room moment simple behave".to_string();
    /// let result = Mnemonic::from_string(str_mnemonic.clone()).unwrap();
    /// assert_eq!(str_mnemonic, result.as_string());
    /// ```
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }

    /// serializes a `Mnemonic` to a vector of words.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use andromeda_bitcoin::mnemonic::Mnemonic;
    ///
    /// let str_mnemonic = "desk prevent enhance husband hungry idle member vessel room moment simple behave".to_string();
    /// let result = Mnemonic::from_string(str_mnemonic.clone()).unwrap();
    /// assert_eq!(result.as_words(), vec!["desk", "prevent", "enhance", "husband", "hungry", "idle", "member", "vessel", "room", "moment", "simple", "behave"]);
    /// ```
    pub fn as_words(&self) -> Vec<String> {
        self.inner.words().map(String::from).collect()
    }

    pub fn inner(&self) -> BdkMnemonic {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use bdk_wallet::keys::bip39::{Error as Bip39Error, Language};

    use super::{get_words_autocomplete, Mnemonic};
    use crate::error::Error;

    #[test]
    fn should_return_match_words_in_english() {
        assert_eq!(
            get_words_autocomplete("can".to_string()),
            vec![
                String::from("can"),
                String::from("canal"),
                String::from("cancel"),
                String::from("candy"),
                String::from("cannon"),
                String::from("canoe"),
                String::from("canvas"),
                String::from("canyon"),
            ]
        );

        assert_eq!(
            get_words_autocomplete("major".to_string()),
            vec![String::from("major"),]
        );
    }

    #[test]
    fn should_return_empty_vector() {
        assert!(get_words_autocomplete("canb".to_string()).is_empty());
    }

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
            Error::Bip39(Bip39Error::UnknownWord(word_index)) => word_index == 0,
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
            Error::Bip39(Bip39Error::UnknownWord(word_index)) => word_index == 0,
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
            Error::Bip39(Bip39Error::BadWordCount(word_index)) => word_index == 11,
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
            Error::Bip39(Bip39Error::UnknownWord(word_index)) => word_index == 0,
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
            mnemonic.as_words(),
            vec![
                "affair", "recycle", "please", "start", "moment", "film", "grain", "myself", "flight", "issue",
                "artwork", "silver"
            ]
        );
    }
}
