use crate::Network;

use bdk::bitcoin::bip32::DerivationPath as BdkDerivationPath;
use bdk::bitcoin::key::Secp256k1;
use bdk::keys::{
    DescriptorPublicKey as BdkDescriptorPublicKey, DescriptorSecretKey as BdkDescriptorSecretKey, ExtendedKey,
};
use bdk::miniscript::descriptor::{DescriptorXKey, Wildcard};
use bdk::Error as BdkError;

use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::mnemonic::Mnemonic;
use bdk::keys::DerivableKey;

pub struct DerivationPath {
    inner_mutex: Mutex<BdkDerivationPath>,
}

impl DerivationPath {
    pub fn new(path: String) -> Result<Self, BdkError> {
        BdkDerivationPath::from_str(&path)
            .map(|x| DerivationPath {
                inner_mutex: Mutex::new(x),
            })
            .map_err(|e| BdkError::Generic(e.to_string()))
    }
}

#[derive(Debug)]
// #[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct DescriptorSecretKey {
    pub inner: BdkDescriptorSecretKey,
}

// #[cfg(feature = "wasm")]
// impl<'de> Deserialize<'de> for DescriptorSecretKey {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserialize<'de>,
//     {
//         // Deserialize the inner string.
//         let s = String::deserialize(deserializer)?;
//         // Ok(DescriptorSecretKey(s))
//     }
// }

impl DescriptorSecretKey {
    pub fn new(network: Network, mnemonic: Arc<Mnemonic>, password: Option<String>) -> Self {
        let mnemonic = mnemonic.inner();
        let xkey: ExtendedKey = (mnemonic, password).into_extended_key().unwrap();
        let descriptor_secret_key = BdkDescriptorSecretKey::XPrv(DescriptorXKey {
            origin: None,
            xkey: xkey.into_xprv(network.into()).unwrap(),
            derivation_path: BdkDerivationPath::master(),
            wildcard: Wildcard::Unhardened,
        });
        Self {
            inner: descriptor_secret_key,
        }
    }

    // pub fn into_inner(self) -> Result<BdkDescriptorSecretKey, BdkError> {
    //     // Attempt to convert the string into a BdkDescriptorSecretKey.
    //     BdkDescriptorSecretKey::from_str(&self.0)
    //         .map_err(|_| BdkError::Generic("Failed to parse DescriptorSecretKey".to_string()))
    // }

    pub fn from_string(private_key: String) -> Result<Self, BdkError> {
        let descriptor_secret_key =
            BdkDescriptorSecretKey::from_str(private_key.as_str()).map_err(|e| BdkError::Generic(e.to_string()))?;
        Ok(Self {
            inner: descriptor_secret_key,
        })
    }

    pub fn derive(&self, path: Arc<DerivationPath>) -> Result<Arc<Self>, BdkError> {
        let secp = Secp256k1::new();
        let descriptor_secret_key = &self.inner;
        let path = path.inner_mutex.lock().unwrap().deref().clone();
        match descriptor_secret_key {
            BdkDescriptorSecretKey::Single(_) => Err(BdkError::Generic("Cannot derive from a single key".to_string())),
            BdkDescriptorSecretKey::XPrv(descriptor_x_key) => {
                let derived_xprv = descriptor_x_key.xkey.derive_priv(&secp, &path)?;
                let key_source = match descriptor_x_key.origin.clone() {
                    Some((fingerprint, origin_path)) => (fingerprint, origin_path.extend(path)),
                    None => (descriptor_x_key.xkey.fingerprint(&secp), path),
                };
                let derived_descriptor_secret_key = BdkDescriptorSecretKey::XPrv(DescriptorXKey {
                    origin: Some(key_source),
                    xkey: derived_xprv,
                    derivation_path: BdkDerivationPath::default(),
                    wildcard: descriptor_x_key.wildcard,
                });
                Ok(Arc::new(Self {
                    inner: derived_descriptor_secret_key,
                }))
            }
            BdkDescriptorSecretKey::MultiXPrv(_) => {
                Err(BdkError::Generic("Cannot derive from a multi key".to_string()))
            }
        }
    }

    pub fn extend(&self, path: Arc<DerivationPath>) -> Result<Arc<Self>, BdkError> {
        let descriptor_secret_key = &self.inner;
        let path = path.inner_mutex.lock().unwrap().deref().clone();
        match descriptor_secret_key {
            BdkDescriptorSecretKey::Single(_) => Err(BdkError::Generic("Cannot extend from a single key".to_string())),
            BdkDescriptorSecretKey::XPrv(descriptor_x_key) => {
                let extended_path = descriptor_x_key.derivation_path.extend(path);
                let extended_descriptor_secret_key = BdkDescriptorSecretKey::XPrv(DescriptorXKey {
                    origin: descriptor_x_key.origin.clone(),
                    xkey: descriptor_x_key.xkey,
                    derivation_path: extended_path,
                    wildcard: descriptor_x_key.wildcard,
                });
                Ok(Arc::new(Self {
                    inner: extended_descriptor_secret_key,
                }))
            }
            BdkDescriptorSecretKey::MultiXPrv(_) => {
                Err(BdkError::Generic("Cannot derive from a multi key".to_string()))
            }
        }
    }

    pub fn as_public(&self) -> Arc<DescriptorPublicKey> {
        let secp = Secp256k1::new();
        let descriptor_public_key = self.inner.to_public(&secp).unwrap();
        Arc::new(DescriptorPublicKey {
            inner: descriptor_public_key,
        })
    }

    /// Get the private key as bytes.
    pub fn secret_bytes(&self) -> Vec<u8> {
        let inner = &self.inner;
        let secret_bytes: Vec<u8> = match inner {
            BdkDescriptorSecretKey::Single(_) => {
                unreachable!()
            }
            BdkDescriptorSecretKey::XPrv(descriptor_x_key) => descriptor_x_key.xkey.private_key.secret_bytes().to_vec(),
            BdkDescriptorSecretKey::MultiXPrv(_) => {
                unreachable!()
            }
        };

        secret_bytes
    }

    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }

    // #[cfg(feature = "wasm")]
    // pub fn from_js_value(js_value: JsValue) -> Result<Self, serde_wasm_bindgen::Error> {
    //     serde_wasm_bindgen::from_value(js_value)
    // }
}

#[derive(Debug)]
pub struct DescriptorPublicKey {
    pub inner: BdkDescriptorPublicKey,
}

impl DescriptorPublicKey {
    pub fn from_string(public_key: String) -> Result<Self, BdkError> {
        let descriptor_public_key =
            BdkDescriptorPublicKey::from_str(public_key.as_str()).map_err(|e| BdkError::Generic(e.to_string()))?;
        Ok(Self {
            inner: descriptor_public_key,
        })
    }

    pub fn derive(&self, path: Arc<DerivationPath>) -> Result<Arc<Self>, BdkError> {
        let secp = Secp256k1::new();
        let descriptor_public_key = &self.inner;
        let path = path.inner_mutex.lock().unwrap().deref().clone();

        match descriptor_public_key {
            BdkDescriptorPublicKey::Single(_) => Err(BdkError::Generic("Cannot derive from a single key".to_string())),
            BdkDescriptorPublicKey::XPub(descriptor_x_key) => {
                let derived_xpub = descriptor_x_key.xkey.derive_pub(&secp, &path)?;
                let key_source = match descriptor_x_key.origin.clone() {
                    Some((fingerprint, origin_path)) => (fingerprint, origin_path.extend(path)),
                    None => (descriptor_x_key.xkey.fingerprint(), path),
                };
                let derived_descriptor_public_key = BdkDescriptorPublicKey::XPub(DescriptorXKey {
                    origin: Some(key_source),
                    xkey: derived_xpub,
                    derivation_path: BdkDerivationPath::default(),
                    wildcard: descriptor_x_key.wildcard,
                });
                Ok(Arc::new(Self {
                    inner: derived_descriptor_public_key,
                }))
            }
            BdkDescriptorPublicKey::MultiXPub(_) => {
                Err(BdkError::Generic("Cannot derive from a multi xpub".to_string()))
            }
        }
    }

    pub fn extend(&self, path: Arc<DerivationPath>) -> Result<Arc<Self>, BdkError> {
        let descriptor_public_key = &self.inner;
        let path = path.inner_mutex.lock().unwrap().deref().clone();
        match descriptor_public_key {
            BdkDescriptorPublicKey::Single(_) => Err(BdkError::Generic("Cannot extend from a single key".to_string())),
            BdkDescriptorPublicKey::XPub(descriptor_x_key) => {
                let extended_path = descriptor_x_key.derivation_path.extend(path);
                let extended_descriptor_public_key = BdkDescriptorPublicKey::XPub(DescriptorXKey {
                    origin: descriptor_x_key.origin.clone(),
                    xkey: descriptor_x_key.xkey,
                    derivation_path: extended_path,
                    wildcard: descriptor_x_key.wildcard,
                });
                Ok(Arc::new(Self {
                    inner: extended_descriptor_public_key,
                }))
            }
            BdkDescriptorPublicKey::MultiXPub(_) => {
                Err(BdkError::Generic("Cannot derive from a multi xpub".to_string()))
            }
        }
    }

    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}
//
// // The goal of these tests to to ensure `bdk-ffi` intermediate code correctly calls `bdk` APIs.
// // These tests should not be used to verify `bdk` behavior that is already tested in the `bdk`
// // crate.
#[cfg(test)]
mod test {
    use crate::keys::{DerivationPath, DescriptorPublicKey, DescriptorSecretKey, Mnemonic};
    use crate::BdkError;
    // use bdk::bitcoin::hashes::hex::ToHex;
    use bdk::bitcoin::Network;
    use std::sync::Arc;

    fn get_inner() -> DescriptorSecretKey {
        let mnemonic = Mnemonic::from_string("chaos fabric time speed sponsor all flat solution wisdom trophy crack object robot pave observe combine where aware bench orient secret primary cable detect".to_string()).unwrap();
        DescriptorSecretKey::new(Network::Testnet.into(), Arc::new(mnemonic), None)
    }

    fn derive_dsk(key: &DescriptorSecretKey, path: &str) -> Result<Arc<DescriptorSecretKey>, BdkError> {
        let path = Arc::new(DerivationPath::new(path.to_string()).unwrap());
        key.derive(path)
    }

    fn extend_dsk(key: &DescriptorSecretKey, path: &str) -> Result<Arc<DescriptorSecretKey>, BdkError> {
        let path = Arc::new(DerivationPath::new(path.to_string()).unwrap());
        key.extend(path)
    }

    fn derive_dpk(key: &DescriptorPublicKey, path: &str) -> Result<Arc<DescriptorPublicKey>, BdkError> {
        let path = Arc::new(DerivationPath::new(path.to_string()).unwrap());
        key.derive(path)
    }

    fn extend_dpk(key: &DescriptorPublicKey, path: &str) -> Result<Arc<DescriptorPublicKey>, BdkError> {
        let path = Arc::new(DerivationPath::new(path.to_string()).unwrap());
        key.extend(path)
    }

    #[test]
    fn test_generate_descriptor_secret_key() {
        let master_dsk = get_inner();
        assert_eq!(master_dsk.as_string(), "tprv8ZgxMBicQKsPdWuqM1t1CDRvQtQuBPyfL6GbhQwtxDKgUAVPbxmj71pRA8raTqLrec5LyTs5TqCxdABcZr77bt2KyWA5bizJHnC4g4ysm4h/*");
        assert_eq!(master_dsk.as_public().as_string(), "tpubD6NzVbkrYhZ4WywdEfYbbd62yuvqLjAZuPsNyvzCNV85JekAEMbKHWSHLF9h3j45SxewXDcLv328B1SEZrxg4iwGfmdt1pDFjZiTkGiFqGa/*");
    }

    #[test]
    fn test_derive_self() {
        let master_dsk = get_inner();
        let derived_dsk: &DescriptorSecretKey = &derive_dsk(&master_dsk, "m").unwrap();
        assert_eq!(derived_dsk.as_string(), "[d1d04177]tprv8ZgxMBicQKsPdWuqM1t1CDRvQtQuBPyfL6GbhQwtxDKgUAVPbxmj71pRA8raTqLrec5LyTs5TqCxdABcZr77bt2KyWA5bizJHnC4g4ysm4h/*");
        let master_dpk: &DescriptorPublicKey = &master_dsk.as_public();
        let derived_dpk: &DescriptorPublicKey = &derive_dpk(master_dpk, "m").unwrap();
        assert_eq!(derived_dpk.as_string(), "[d1d04177]tpubD6NzVbkrYhZ4WywdEfYbbd62yuvqLjAZuPsNyvzCNV85JekAEMbKHWSHLF9h3j45SxewXDcLv328B1SEZrxg4iwGfmdt1pDFjZiTkGiFqGa/*");
    }

    #[test]
    fn test_derive_descriptors_keys() {
        let master_dsk = get_inner();
        let derived_dsk: &DescriptorSecretKey = &derive_dsk(&master_dsk, "m/0").unwrap();
        assert_eq!(derived_dsk.as_string(), "[d1d04177/0]tprv8d7Y4JLmD25jkKbyDZXcdoPHu1YtMHuH21qeN7mFpjfumtSU7eZimFYUCSa3MYzkEYfSNRBV34GEr2QXwZCMYRZ7M1g6PUtiLhbJhBZEGYJ/*");
        let master_dpk: &DescriptorPublicKey = &master_dsk.as_public();
        let derived_dpk: &DescriptorPublicKey = &derive_dpk(master_dpk, "m/0").unwrap();
        assert_eq!(derived_dpk.as_string(), "[d1d04177/0]tpubD9oaCiP1MPmQdndm7DCD3D3QU34pWd6BbKSRedoZF1UJcNhEk3PJwkALNYkhxeTKL29oGNR7psqvT1KZydCGqUDEKXN6dVQJY2R8ooLPy8m/*");
    }

    #[test]
    fn test_extend_descriptor_keys() {
        let master_dsk = get_inner();
        let extended_dsk: &DescriptorSecretKey = &extend_dsk(&master_dsk, "m/0").unwrap();
        assert_eq!(extended_dsk.as_string(), "tprv8ZgxMBicQKsPdWuqM1t1CDRvQtQuBPyfL6GbhQwtxDKgUAVPbxmj71pRA8raTqLrec5LyTs5TqCxdABcZr77bt2KyWA5bizJHnC4g4ysm4h/0/*");
        let master_dpk: &DescriptorPublicKey = &master_dsk.as_public();
        let extended_dpk: &DescriptorPublicKey = &extend_dpk(master_dpk, "m/0").unwrap();
        assert_eq!(extended_dpk.as_string(), "tpubD6NzVbkrYhZ4WywdEfYbbd62yuvqLjAZuPsNyvzCNV85JekAEMbKHWSHLF9h3j45SxewXDcLv328B1SEZrxg4iwGfmdt1pDFjZiTkGiFqGa/0/*");
        let wif = "L2wTu6hQrnDMiFNWA5na6jB12ErGQqtXwqpSL7aWquJaZG8Ai3ch";
        let extended_key = DescriptorSecretKey::from_string(wif.to_string()).unwrap();
        let result = extended_key.derive(Arc::new(DerivationPath::new("m/0".to_string()).unwrap()));
        dbg!(&result);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_inner() {
        let key1 = "L2wTu6hQrnDMiFNWA5na6jB12ErGQqtXwqpSL7aWquJaZG8Ai3ch";
        let key2 = "tprv8ZgxMBicQKsPcwcD4gSnMti126ZiETsuX7qwrtMypr6FBwAP65puFn4v6c3jrN9VwtMRMph6nyT63NrfUL4C3nBzPcduzVSuHD7zbX2JKVc/1/1/1/*";
        let private_descriptor_key1 = DescriptorSecretKey::from_string(key1.to_string()).unwrap();
        let private_descriptor_key2 = DescriptorSecretKey::from_string(key2.to_string()).unwrap();
        dbg!(private_descriptor_key1);
        dbg!(private_descriptor_key2);
        // Should error out because you can't produce a DescriptorSecretKey from an xpub
        let key0 = "tpubDBrgjcxBxnXyL575sHdkpKohWu5qHKoQ7TJXKNrYznh5fVEGBv89hA8ENW7A8MFVpFUSvgLqc4Nj1WZcpePX6rrxviVtPowvMuGF5rdT2Vi";
        assert!(DescriptorSecretKey::from_string(key0.to_string()).is_err());
    }

    #[test]
    fn test_derive_and_extend_inner() {
        let master_dsk = get_inner();
        // derive DescriptorSecretKey with path "m/0" from master
        let derived_dsk: &DescriptorSecretKey = &derive_dsk(&master_dsk, "m/0").unwrap();
        assert_eq!(derived_dsk.as_string(), "[d1d04177/0]tprv8d7Y4JLmD25jkKbyDZXcdoPHu1YtMHuH21qeN7mFpjfumtSU7eZimFYUCSa3MYzkEYfSNRBV34GEr2QXwZCMYRZ7M1g6PUtiLhbJhBZEGYJ/*");
        // extend derived_dsk with path "m/0"
        let extended_dsk: &DescriptorSecretKey = &extend_dsk(derived_dsk, "m/0").unwrap();
        assert_eq!(extended_dsk.as_string(), "[d1d04177/0]tprv8d7Y4JLmD25jkKbyDZXcdoPHu1YtMHuH21qeN7mFpjfumtSU7eZimFYUCSa3MYzkEYfSNRBV34GEr2QXwZCMYRZ7M1g6PUtiLhbJhBZEGYJ/0/*");
    }

    #[test]
    fn test_derive_hardened_path_using_public() {
        let master_dpk: Arc<DescriptorPublicKey> = get_inner().as_public();
        let derived_dpk = &derive_dpk(&master_dpk, "m/84h/1h/0h");
        assert!(derived_dpk.is_err());
    }

    // TODO 7: It appears that the to_hex() method is not available anymore.
    //       Look into the correct way to pull the hex out of the DescriptorSecretKey.
    //       Note: ToHex was removed in bitcoin_hashes 0.12.0
    // #[test]
    // fn test_retrieve_master_secret_key() {
    //     let master_dpk = get_inner();
    //     let master_private_key = master_dpk.secret_bytes().to_hex();
    //     assert_eq!(
    //         master_private_key,
    //         "e93315d6ce401eb4db803a56232f0ed3e69b053774e6047df54f1bd00e5ea936"
    //     )
    // }
}

// /// Generate a new mnemonic phrase
// pub fn gen_mnemonic(count: WordCount) -> String {
//     // Generate fresh mnemonic
//     let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((count, Language::English)).unwrap();
//     // Convert mnemonic to string
//     let mnemonic_words = mnemonic.to_string();
//     return mnemonic_words;
// }

pub fn gen_address(mnemonic: &str, passphrase: &str, path: &str, _network: Network) -> String {
    println!("mnemonic: {}\n", mnemonic);
    println!("passphrase: {}\n", passphrase);
    println!("path: {}\n", path);
    return "addressOne".to_string();
}
