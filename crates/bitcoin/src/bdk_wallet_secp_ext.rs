use std::str::FromStr;

use bdk_wallet::{keys::DescriptorSecretKey, Wallet};
use bitcoin::{bip32::DerivationPath, secp256k1, Address};

use crate::error::Error;

pub(crate) trait BdkWalletSecpExt {
    fn get_secp256k1_keypair(&self, address: &str) -> Result<(secp256k1::Keypair, bool), Error>;
    fn get_wif_string(&self) -> Result<String, Error>;
}

impl BdkWalletSecpExt for Wallet {
    /// Returns a `secp256k1::Keypair`
    /// the underlying Keypair and a boolean indicating whether the key is compressed.
    ///
    /// # Arguments
    ///
    /// * `address` - A Bitcoin address (string) that must match the network of this wallet.
    ///
    /// # Returns
    ///
    /// * `Ok((secp256k1::Keypair, bool))` on success, or an `Error` variant on failure.
    ///
    /// The `bool` indicates if the derived private key is using compressed pubkeys.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidNetwork`] if `address` does not match the wallet's network
    /// * [`Error::InvalidAddress`] if the script pubkey cannot be found in the wallet
    /// * [`Error::NoSignerFound`] if the wallet has no signers or the signer is invalid
    /// * [`Error::InvalidNetwork`] if the address is mismatched to the wallet network
    fn get_secp256k1_keypair(&self, address: &str) -> Result<(secp256k1::Keypair, bool), Error> {
        // 1) Parse the address and ensure the network matches
        let spk = Address::from_str(address)?
            .require_network(self.network())
            .map_err(|_| Error::InvalidNetwork)?
            .script_pubkey();

        // 2) Locate the scriptPubKey's derivation index in the wallet
        let (keychain_kind, spk_index) = match self.derivation_of_spk(spk) {
            Some((keychain_kind, spk_index)) => (keychain_kind, spk_index),
            None => return Err(Error::InvalidAddress(address.to_owned())),
        };

        // 3) Get the signer
        let secp = self.secp_ctx();
        let signers_container = self.get_signers(keychain_kind);
        let signers = signers_container.signers();
        let signer = signers.first().ok_or(Error::NoSignerFound)?;

        // 4) Extract the descriptor secret key (xprv) from the signer
        let xkey = match signer.descriptor_secret_key() {
            Some(DescriptorSecretKey::XPrv(xkey)) => xkey,
            _ => return Err(Error::NoSignerFound),
        };

        // 5) Derive the private key from the xprv at "0/spk_index"
        let derivation_path_for_subkey = DerivationPath::from_str(&format!("0/{}", spk_index))?;
        let derived_key = xkey.xkey.derive_priv(secp, &derivation_path_for_subkey)?;
        let derived_keypair = secp256k1::Keypair::from_secret_key(secp, &derived_key.private_key);
        let is_compressed = derived_key.to_priv().compressed;

        // return
        Ok((derived_keypair, is_compressed))
    }

    fn get_wif_string(&self) -> Result<String, Error> {
        // 1) Get the signer
        let signers_container = self.get_signers(bdk_wallet::KeychainKind::External);
        let signers = signers_container.signers();
        let signer = signers.first().ok_or(Error::NoSignerFound)?;
        // 2) Extract the descriptor secret key (xprv) from the signer
        let xkey = match signer.descriptor_secret_key() {
            Some(DescriptorSecretKey::Single(xkey)) => xkey,
            _ => return Err(Error::NoSignerKeyFound),
        };
        Ok(xkey.key.to_wif())
    }
}
