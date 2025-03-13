#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use andromeda_common::{Network, ScriptType};
    use bitcoin::{
        bip32::{DerivationPath, Xpriv},
        key::Secp256k1,
        NetworkKind,
    };

    use crate::{
        account::Account,
        mnemonic::Mnemonic,
        storage::{WalletFilePersisterFactory, WalletMemoryPersisterFactory, WalletPersisterFactory, WalletStorage},
    };

    /// Creates a test wallet account using the provided mnemonic, derivation path, and optional storage settings.
    ///
    /// # Arguments
    ///
    /// - `mnemonic` - A string slice containing the mnemonic phrase for generating the wallet.
    /// - `script_type` - The type of script used (e.g., Legacy, SegWit, Taproot).
    /// - `derivation_path` - A string slice representing the derivation path (e.g., "m/44'/0'/0'/0").
    /// - `file_store` - An optional boolean flag indicating whether to store the wallet data in a file.
    ///   - **Default:** `false` (uses in-memory storage).
    /// - `file_read_only` - An optional boolean flag to mark the file storage as read-only.  
    ///   - **Default:** `false` (file storage is writable if enabled).
    /// - `network` - An optional `Network` enum specifying the network (e.g., Regtest, Mainnet, Testnet).  
    ///   - **Default:** `Network::Regtest`.
    /// - `network_kind` - An optional `NetworkKind` enum defining the type of network (e.g., Test, Main).  
    ///   - **Default:** `NetworkKind::Test`.
    ///
    /// # Returns
    ///
    /// - Returns an `Account` instance representing the created test wallet.
    ///
    /// # Panics
    ///
    /// - Panics if the mnemonic is invalid or if account creation fails.
    ///
    /// # Example Usage
    ///
    /// ```rust
    /// let test_account = set_test_wallet_account(
    ///     "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ///     ScriptType::NativeSegWit,
    ///     "m/84'/0'/0'/0",
    ///     Some(true),   // Enable file storage
    ///     Some(false),  // File storage is writable
    ///     Some(Network::Regtest), // Use Regtest
    ///     Some(NetworkKind::Test), // Use Test network kind
    /// );
    /// ```
    pub fn set_test_wallet_account(
        mnemonic: &str,
        script_type: ScriptType,
        derivation_path: &str,
        file_store: Option<bool>,
        file_read_only: Option<bool>,
        network: Option<Network>,
        network_kind: Option<NetworkKind>,
    ) -> Account {
        let mnemonic = Mnemonic::from_string(mnemonic.to_string()).unwrap();
        let master_secret_key =
            Xpriv::new_master(network_kind.unwrap_or(NetworkKind::Test), &mnemonic.inner().to_seed("")).unwrap();
        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        let account_network = network.unwrap_or(Network::Regtest);
        let secp = Secp256k1::new();
        let store_key = format!(
            "{}_{}_{}",
            account_network,
            master_secret_key.fingerprint(&secp),
            derivation_path
        );
        let clean_key = store_key.replace("'", "_").replace("/", "_");

        let factory = if file_store.unwrap_or(false) {
            WalletFilePersisterFactory(file_read_only.unwrap_or(false)).build(clean_key)
        } else {
            WalletMemoryPersisterFactory.build(clean_key)
        };

        Account::new(
            master_secret_key,
            account_network,
            script_type,
            derivation_path,
            WalletStorage(factory),
        )
        .unwrap()
    }
}
