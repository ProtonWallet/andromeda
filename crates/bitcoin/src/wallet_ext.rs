use bdk_wallet::{KeychainKind, Wallet};

pub trait BdkWalletExt {
    fn mark_used_to(&mut self, keychain: KeychainKind, from: u32, to: Option<u32>);
}

impl BdkWalletExt for Wallet {
    /// Marks a range of addresses as used in the specified keychain.
    ///
    /// This method allows you to mark multiple consecutive addresses as used,
    /// starting from a specified index and ending at either a given index
    /// or the next index if the end is not provided.
    ///
    /// # Arguments
    ///
    /// * `keychain` - The `KeychainKind` indicating which keychain to update
    ///   (e.g., External or Internal).
    /// * `from` - The starting index of the range to mark as used. This is a
    ///   required parameter.
    /// * `to` - An optional ending index of the range. If `None`, only the
    ///   starting index `from` will be marked as used. If `Some(end_index)` is
    ///   provided, all indices from `from` to `end_index - 1` will be marked as
    ///   used.
    ///
    /// # Example
    ///
    /// ```rust
    /// use andromeda_bitcoin::wallet_ext::BdkWalletExt;
    /// use bdk_wallet::{bitcoin::{Network, secp256k1::Secp256k1}, KeychainKind, Wallet};
    /// use miniscript::descriptor::Descriptor;
    ///
    /// let secp = Secp256k1::new();
    /// let (external_descriptor, _external_keymap) = Descriptor::parse_descriptor(&secp, "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/0/*)").unwrap();
    /// let (internal_descriptor, _internal_keymap) = Descriptor::parse_descriptor(&secp, "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/1/*)").unwrap();
    /// let mut wallet = Wallet::new(external_descriptor, internal_descriptor, Network::Testnet).unwrap();
    ///
    /// // This will mark addresses 10, 11, 12, 13, and 14 as used in the External keychain.
    /// wallet.mark_used_to(KeychainKind::External, 10, Some(15));
    /// // This will mark only address 20 as used in the Internal keychain.
    /// wallet.mark_used_to(KeychainKind::Internal, 20, None);
    /// ```
    ///
    /// # Note
    ///
    /// If the `mark_used_to` method is called with `to < from`, it will
    /// constrain `to` to `from + 1`
    fn mark_used_to(&mut self, keychain: KeychainKind, from: u32, to: Option<u32>) {
        let to = to.unwrap_or(from + 1);
        let to = if from < to { to } else { from + 1 };

        // Make sure indexes are revealed up to upper limit, else mark_used won't have
        // any effect
        let _ = self.reveal_addresses_to(keychain, to);

        for index in from..to {
            self.mark_used(keychain, index);
        }
    }
}
