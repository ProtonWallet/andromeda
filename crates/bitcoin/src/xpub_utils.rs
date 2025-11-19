//! Extended public key (xpub) version conversion utilities.
//!
//! This module provides utilities for converting between different extended public key formats
//! (xpub, ypub, zpub, tpub, upub, vpub) by manipulating their version bytes. These different
//! formats indicate different derivation paths and address types:
//!
//! - **xpub/tpub**: Legacy P2PKH or P2SH addresses (BIP44/BIP45)
//! - **ypub/upub**: SegWit wrapped in P2SH addresses (BIP49)
//! - **zpub/vpub**: Native SegWit P2WPKH addresses (BIP84)
//!
//! Mainnet versions use xpub/ypub/zpub prefixes, while testnet uses tpub/upub/vpub.
//!
//! # Examples
//!
//! ```
//! use andromeda_bitcoin::xpub_utils::{convert_version, Version};
//!
//! let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
//! let zpub = convert_version(xpub, &Version::Zpub).unwrap();
//! ```

use crate::error::Error;
use crate::error::Error::{BadChecksum, InvalidBase58Char, InvalidExtendedPublicKey, InvalidLength};
use bitcoin::base58;
use std::str::FromStr;

pub const EXTENDED_KEY_LENGTH: usize = 78;

/// Version bytes xpub: bitcoin mainnet public key P2PKH or P2SH
pub const VERSION_XPUB: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];
/// Version bytes ypub: bitcoin mainnet public key P2WPKH in P2SH
pub const VERSION_YPUB: [u8; 4] = [0x04, 0x9D, 0x7C, 0xB2];
/// Version bytes zpub: bitcoin mainnet public key P2WPKH
pub const VERSION_ZPUB: [u8; 4] = [0x04, 0xB2, 0x47, 0x46];
/// Version bytes tpub: bitcoin testnet public key for P2PKH or P2SH
pub const VERSION_TPUB: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];
/// Version bytes upub: bitcoin testnet public key for P2WPKH in P2SH
pub const VERSION_UPUB: [u8; 4] = [0x04, 0x4A, 0x52, 0x62];
/// Version bytes vpub: bitcoin testnet public key for P2WPKH
pub const VERSION_VPUB: [u8; 4] = [0x04, 0x5F, 0x1C, 0xF6];

impl From<base58::Error> for Error {
    fn from(err: base58::Error) -> Self {
        match err {
            base58::Error::Decode(err) => InvalidBase58Char(err.invalid_base58_character()),
            base58::Error::TooShort(err) => InvalidLength(err.invalid_base58_length()),
            base58::Error::IncorrectChecksum(err) => {
                let (expected, actual) = err.incorrect_checksum();
                BadChecksum { expected, actual }
            }
            _ => InvalidExtendedPublicKey,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// Version bytes xpub: bitcoin mainnet public key P2PKH or P2SH (BIP44/45)
    Xpub,
    /// Version bytes ypub: bitcoin mainnet public key P2WPKH in P2SH (BIP49)
    Ypub,
    /// Version bytes zpub: bitcoin mainnet public key P2WPKH (BIP84)
    Zpub,
    /// Version bytes tpub: bitcoin testnet public key for P2PKH or P2SH
    Tpub,
    /// Version bytes upub: bitcoin testnet public key for P2WPKH in P2SH
    Upub,
    /// Version bytes vpub: bitcoin testnet public key for P2WPKH
    Vpub,
}

impl Version {
    pub const fn bytes(&self) -> [u8; 4] {
        match self {
            Version::Xpub => VERSION_XPUB,
            Version::Ypub => VERSION_YPUB,
            Version::Zpub => VERSION_ZPUB,
            Version::Tpub => VERSION_TPUB,
            Version::Upub => VERSION_UPUB,
            Version::Vpub => VERSION_VPUB,
        }
    }

    /// Attempts to detect the version from the first 4 bytes of an extended key.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let prefix: [u8; 4] = bytes[0..4].try_into().ok()?;

        match prefix {
            VERSION_XPUB => Some(Version::Xpub),
            VERSION_YPUB => Some(Version::Ypub),
            VERSION_ZPUB => Some(Version::Zpub),
            VERSION_TPUB => Some(Version::Tpub),
            VERSION_UPUB => Some(Version::Upub),
            VERSION_VPUB => Some(Version::Vpub),
            _ => None,
        }
    }
}

impl FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xpub" => Ok(Version::Xpub),
            "ypub" => Ok(Version::Ypub),
            "zpub" => Ok(Version::Zpub),
            "tpub" => Ok(Version::Tpub),
            "upub" => Ok(Version::Upub),
            "vpub" => Ok(Version::Vpub),
            _ => Err(Self::Err::UnknownVersionPrefix),
        }
    }
}

/// Replaces the first 4 bytes of a byte slice with the target's version and returns a new byte vec.
///
/// This function validates that the input is exactly 78 bytes (the standard BIP32 extended key length)
/// to ensure it's a valid extended public/private key before replacing the version bytes.
///
/// # Examples
///
/// ```
/// use andromeda_bitcoin::xpub_utils::{replace_version_bytes, Version};
///
/// let xpub_bytes = vec![0u8; 78]; // Example extended key bytes
/// let result = replace_version_bytes(&xpub_bytes, &Version::Zpub);
/// ```
pub fn replace_version_bytes<B: AsRef<[u8]>>(bytes: B, target: &Version) -> Result<Vec<u8>, Error> {
    let mut vec = bytes.as_ref().to_vec();

    if vec.len() != EXTENDED_KEY_LENGTH {
        return Err(Error::InvalidLength(vec.len()));
    }

    vec[0..4].copy_from_slice(&target.bytes());

    Ok(vec)
}

/// Converts an extended public key from one version format to another.
///
/// # Examples
///
/// ```
/// use andromeda_bitcoin::xpub_utils::{convert_version, Version};
///
/// let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
/// let zpub = convert_version(xpub, &Version::Zpub).unwrap();
/// assert_eq!(zpub, "zpub6qUQGY8YyN3ZxYEgf8J6KCQBqQAbdSWaT9RK54L5FWTTh8na8NkCkZpYHnWt7zEwNhqd6p9Utq562cSZsqGqFE87NNsUKnyZeJ5KvbhfC8E");
/// ```
pub fn convert_version<S: AsRef<str>>(str: S, target: &Version) -> Result<String, Error> {
    let bytes = base58::decode_check(str.as_ref())?;
    let replaced = replace_version_bytes(bytes, target)?;

    Ok(base58::encode_check(&replaced))
}

/// Automatically converts an extended public key to the appropriate format based on script type and network.
///
/// This function converts:
/// - Native SegWit (BIP84) → zpub (mainnet) or vpub (testnet/signet/regtest)
/// - Nested SegWit (BIP49) → ypub (mainnet) or upub (testnet/signet/regtest)
/// - Legacy/Taproot → xpub (mainnet) or tpub (testnet/signet/regtest)
///
/// # Examples
///
/// ```
/// use andromeda_bitcoin::xpub_utils::convert_to_script_type;
/// use andromeda_common::{ScriptType, Network};
///
/// let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
///
/// // Convert to native segwit format (zpub)
/// let zpub = convert_to_script_type(xpub, ScriptType::NativeSegwit, Network::Bitcoin).unwrap();
/// assert_eq!(zpub, "zpub6qUQGY8YyN3ZxYEgf8J6KCQBqQAbdSWaT9RK54L5FWTTh8na8NkCkZpYHnWt7zEwNhqd6p9Utq562cSZsqGqFE87NNsUKnyZeJ5KvbhfC8E");
///
/// // Convert to nested segwit format (ypub)
/// let ypub = convert_to_script_type(xpub, ScriptType::NestedSegwit, Network::Bitcoin).unwrap();
/// assert_eq!(ypub, "ypub6We8xsTdpgW67F3ZpmWU77JgfS29gpX5Y2u6HfSBsW5ae2yLsiae8WAQGaZJ85b1y4ipMLYvSAiY9Kq1A8rpSzSWW3B3jtA5Na1gXzZ8iqF");
///
/// // Legacy stays as xpub
/// let xpub_result = convert_to_script_type(xpub, ScriptType::Legacy, Network::Bitcoin).unwrap();
/// assert_eq!(xpub_result, xpub);
/// ```
pub fn convert_to_script_type<S: AsRef<str>>(
    key: S,
    script_type: andromeda_common::ScriptType,
    network: andromeda_common::Network,
) -> Result<String, Error> {
    use andromeda_common::{Network, ScriptType};

    let target_version = match (script_type, network) {
        (ScriptType::NativeSegwit, Network::Bitcoin) => Version::Zpub,
        (ScriptType::NativeSegwit, _) => Version::Vpub,
        (ScriptType::NestedSegwit, Network::Bitcoin) => Version::Ypub,
        (ScriptType::NestedSegwit, _) => Version::Upub,
        (_, Network::Bitcoin) => Version::Xpub,
        (_, _) => Version::Tpub,
    };

    convert_version(key, &target_version)
}

#[cfg(test)]
mod tests {
    use crate::xpub_utils::*;

    #[test]
    fn test_version_bytes() {
        assert_eq!(Version::Xpub.bytes(), VERSION_XPUB);
        assert_eq!(Version::Ypub.bytes(), VERSION_YPUB);
        assert_eq!(Version::Zpub.bytes(), VERSION_ZPUB);
        assert_eq!(Version::Tpub.bytes(), VERSION_TPUB);
        assert_eq!(Version::Upub.bytes(), VERSION_UPUB);
        assert_eq!(Version::Vpub.bytes(), VERSION_VPUB);
    }

    #[test]
    fn test_version_from_bytes() {
        assert_eq!(Version::from_bytes(&VERSION_XPUB), Some(Version::Xpub));
        assert_eq!(Version::from_bytes(&VERSION_YPUB), Some(Version::Ypub));
        assert_eq!(Version::from_bytes(&VERSION_ZPUB), Some(Version::Zpub));
        assert_eq!(Version::from_bytes(&VERSION_TPUB), Some(Version::Tpub));
        assert_eq!(Version::from_bytes(&VERSION_UPUB), Some(Version::Upub));
        assert_eq!(Version::from_bytes(&VERSION_VPUB), Some(Version::Vpub));
        assert_eq!(Version::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF]), None);
        assert_eq!(Version::from_bytes(&[0x04, 0x88]), None);
    }

    #[test]
    fn test_convert_version_too_short() {
        let short = "abc";
        let result = convert_version(short, &Version::Zpub);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_version_bytes_too_few_bytes() {
        let short = vec![0x35];
        let result = replace_version_bytes(short, &Version::Zpub);
        assert!(matches!(result, Err(Error::InvalidLength(1))));
    }

    #[test]
    fn test_replace_version_bytes_wrong_length() {
        // Test with 77 bytes (one short)
        let short = vec![0u8; 77];
        let result = replace_version_bytes(&short, &Version::Zpub);
        assert!(matches!(result, Err(Error::InvalidLength(77))));

        // Test with 79 bytes (one too many)
        let long = vec![0u8; 79];
        let result = replace_version_bytes(&long, &Version::Zpub);
        assert!(matches!(result, Err(Error::InvalidLength(79))));
    }

    #[test]
    fn test_replace_version_bytes_exact_length() {
        // Test with exactly 78 bytes
        let mut bytes = vec![0u8; EXTENDED_KEY_LENGTH];
        bytes[0..4].copy_from_slice(&VERSION_XPUB);

        let result = replace_version_bytes(&bytes, &Version::Zpub).unwrap();

        assert_eq!(result.len(), EXTENDED_KEY_LENGTH);
        assert_eq!(&result[0..4], &VERSION_ZPUB);
        // Rest should be unchanged
        assert_eq!(&result[4..], &bytes[4..]);
    }

    #[test]
    fn test_xpub_not_valid() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYekkudhUd9yLb6qx39T9nMdj";
        let result = convert_version(xpub, &Version::Ypub);
        assert!(result.is_err());
    }

    #[test]
    fn test_xpub_to_ypub() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_ypub = "ypub6We8xsTdpgW67F3ZpmWU77JgfS29gpX5Y2u6HfSBsW5ae2yLsiae8WAQGaZJ85b1y4ipMLYvSAiY9Kq1A8rpSzSWW3B3jtA5Na1gXzZ8iqF";

        let result = convert_version(xpub, &Version::Ypub).unwrap();
        assert_eq!(result, expected_ypub);
    }

    #[test]
    fn test_xpub_to_zpub() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_zpub = "zpub6qUQGY8YyN3ZxYEgf8J6KCQBqQAbdSWaT9RK54L5FWTTh8na8NkCkZpYHnWt7zEwNhqd6p9Utq562cSZsqGqFE87NNsUKnyZeJ5KvbhfC8E";

        let result = convert_version(xpub, &Version::Zpub).unwrap();
        assert_eq!(result, expected_zpub);
    }

    #[test]
    fn test_zpub_to_xpub() {
        let zpub = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
        let expected_xpub = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

        let result = convert_version(zpub, &Version::Xpub).unwrap();
        assert_eq!(result, expected_xpub);
    }

    #[test]
    fn test_xpub_to_tpub() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_tpub = "tpubDCBWBScQPGv4Xk3JSbhw6wYYpayMjb2eAYyArpbSqQTbLDpphHGAetB6VQgVeftLML8vDSUEWcC2xDi3qJJ3YCDChJDvqVzpgoYSuT52MhJ";

        let result = convert_version(xpub, &Version::Tpub).unwrap();
        assert_eq!(result, expected_tpub);
    }

    #[test]
    fn test_all_testnet_conversions() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        let tpub = convert_version(xpub, &Version::Tpub).unwrap();
        let upub = convert_version(xpub, &Version::Upub).unwrap();
        let vpub = convert_version(xpub, &Version::Vpub).unwrap();

        assert!(tpub.starts_with("tpub"));
        assert!(upub.starts_with("upub"));
        assert!(vpub.starts_with("vpub"));

        let back_from_tpub = convert_version(&tpub, &Version::Xpub).unwrap();
        let back_from_upub = convert_version(&upub, &Version::Xpub).unwrap();
        let back_from_vpub = convert_version(&vpub, &Version::Xpub).unwrap();

        assert_eq!(back_from_tpub, xpub);
        assert_eq!(back_from_upub, xpub);
        assert_eq!(back_from_vpub, xpub);
    }

    #[test]
    fn test_round_trip_all_versions() {
        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        for version in &[
            Version::Xpub,
            Version::Ypub,
            Version::Zpub,
            Version::Tpub,
            Version::Upub,
            Version::Vpub,
        ] {
            let converted = convert_version(xpub, version).unwrap();
            let back = convert_version(&converted, &Version::Xpub).unwrap();
            assert_eq!(back, xpub, "Round trip failed for {:?}", version);
        }
    }

    #[test]
    fn test_convert_to_script_type_native_segwit_mainnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_zpub = "zpub6qUQGY8YyN3ZxYEgf8J6KCQBqQAbdSWaT9RK54L5FWTTh8na8NkCkZpYHnWt7zEwNhqd6p9Utq562cSZsqGqFE87NNsUKnyZeJ5KvbhfC8E";

        let result = convert_to_script_type(xpub, ScriptType::NativeSegwit, Network::Bitcoin).unwrap();
        assert_eq!(result, expected_zpub);
    }

    #[test]
    fn test_convert_to_script_type_nested_segwit_mainnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_ypub = "ypub6We8xsTdpgW67F3ZpmWU77JgfS29gpX5Y2u6HfSBsW5ae2yLsiae8WAQGaZJ85b1y4ipMLYvSAiY9Kq1A8rpSzSWW3B3jtA5Na1gXzZ8iqF";

        let result = convert_to_script_type(xpub, ScriptType::NestedSegwit, Network::Bitcoin).unwrap();
        assert_eq!(result, expected_ypub);
    }

    #[test]
    fn test_convert_to_script_type_legacy_mainnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        let result = convert_to_script_type(xpub, ScriptType::Legacy, Network::Bitcoin).unwrap();
        assert_eq!(result, xpub);
    }

    #[test]
    fn test_convert_to_script_type_taproot_mainnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        // Taproot also uses xpub format
        let result = convert_to_script_type(xpub, ScriptType::Taproot, Network::Bitcoin).unwrap();
        assert_eq!(result, xpub);
    }

    #[test]
    fn test_convert_to_script_type_native_segwit_testnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        let result = convert_to_script_type(xpub, ScriptType::NativeSegwit, Network::Testnet).unwrap();
        assert!(result.starts_with("vpub"));
    }

    #[test]
    fn test_convert_to_script_type_nested_segwit_testnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

        let result = convert_to_script_type(xpub, ScriptType::NestedSegwit, Network::Testnet).unwrap();
        assert!(result.starts_with("upub"));
    }

    #[test]
    fn test_convert_to_script_type_legacy_testnet() {
        use andromeda_common::{Network, ScriptType};

        let xpub = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
        let expected_tpub = "tpubDCBWBScQPGv4Xk3JSbhw6wYYpayMjb2eAYyArpbSqQTbLDpphHGAetB6VQgVeftLML8vDSUEWcC2xDi3qJJ3YCDChJDvqVzpgoYSuT52MhJ";

        let result = convert_to_script_type(xpub, ScriptType::Legacy, Network::Testnet).unwrap();
        assert_eq!(result, expected_tpub);
    }
}
