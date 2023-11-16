use crate::bitcoin::Network;
use crate::error::Error;

use bdk::bitcoin::bip32::DerivationPath;
use bdk::bitcoin::bip32::ExtendedPrivKey;
use bdk::bitcoin::key::Secp256k1;
use bdk::keys::bip39::{Mnemonic as BdkMnemonic, MnemonicWithPassphrase};
use bdk::keys::DerivableKey;
use bdk::keys::{DescriptorSecretKey, ExtendedKey};
use bdk::miniscript::descriptor::DescriptorXKey;
use bdk::miniscript::Segwitv0;
use std::sync::Arc;

pub trait Deriveable {
    fn derive(&self, path: DerivationPath) -> Result<Arc<Self>, Error>;
}

impl Deriveable for DescriptorSecretKey {
    fn derive(&self, path: DerivationPath) -> Result<Arc<Self>, Error> {
        let secp = Secp256k1::new();

        match self {
            DescriptorSecretKey::XPrv(descriptor_x_key) => {
                let derived_xprv = descriptor_x_key.xkey.derive_priv(&secp, &path).map_err(|e| e.into())?;

                let key_source = match descriptor_x_key.origin.clone() {
                    Some((fingerprint, origin_path)) => (fingerprint, origin_path.extend(path)),
                    None => (descriptor_x_key.xkey.fingerprint(&secp), path),
                };

                let derived_descriptor_secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
                    origin: Some(key_source),
                    xkey: derived_xprv,
                    derivation_path: DerivationPath::default(),
                    wildcard: descriptor_x_key.wildcard,
                });

                Ok(Arc::new(derived_descriptor_secret_key))
            }
            DescriptorSecretKey::Single(_) => Err(Error::Generic {
                msg: "Cannot derive from a single key".to_string(),
            }),
            DescriptorSecretKey::MultiXPrv(_) => Err(Error::Generic {
                msg: "Cannot derive from a multi key".to_string(),
            }),
        }
    }
}

pub trait Extendable {
    fn extend(&self, path: DerivationPath) -> Result<Arc<Self>, Error>;
}

impl Extendable for DescriptorSecretKey {
    fn extend(&self, path: DerivationPath) -> Result<Arc<Self>, Error> {
        match self {
            DescriptorSecretKey::XPrv(descriptor_x_key) => {
                let extended_path = descriptor_x_key.derivation_path.extend(path);
                let extended_descriptor_secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
                    origin: descriptor_x_key.origin.clone(),
                    xkey: descriptor_x_key.xkey,
                    derivation_path: extended_path,
                    wildcard: descriptor_x_key.wildcard,
                });
                Ok(Arc::new(extended_descriptor_secret_key))
            }
            DescriptorSecretKey::Single(_) => Err(Error::Generic {
                msg: "Cannot extend from a single key".to_string(),
            }),
            DescriptorSecretKey::MultiXPrv(_) => Err(Error::Generic {
                msg: "Cannot derive from a multi key".to_string(),
            }),
        }
    }
}

// Move this to wallet
pub fn new_master_private_key(mnemonic_str: &str, passphrase: Option<String>) -> ExtendedPrivKey {
    let mnemonic = BdkMnemonic::parse(mnemonic_str).unwrap();
    let mnemonic: MnemonicWithPassphrase = (mnemonic, passphrase);

    let masterxkey: ExtendedKey<Segwitv0> = mnemonic.into_extended_key().unwrap();
    masterxkey.into_xprv(Network::Testnet.into()).unwrap()
}
