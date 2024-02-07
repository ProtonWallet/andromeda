use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use super::{payment_link::PaymentLink, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{bitcoin::Network, transactions::TransactionDetails};
use crate::{error::Error, transactions::SimpleTransaction};
use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    blockchain::esplora::EsploraBlockchain,
    database::BatchDatabase,
    descriptor,
    wallet::{AddressIndex, AddressInfo},
    Balance as BdkBalance, KeychainKind, LocalUtxo, SignOptions, SyncOptions, Wallet,
};
use bitcoin::Transaction;
use miniscript::{
    bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, Address, Network as BdkNetwork, Txid},
    descriptor::DescriptorSecretKey,
    Descriptor, DescriptorPublicKey,
};
use std::fmt::Debug;

/// TLDR; A wallet is defined by its mnemonic + passphrase combo whereas a wallet account is defined by its derivation path from the wallet masterkey.
/// In order to support wallet import from other major softwares, it has been decided to support the BIP44 standard from the very beginning. This BIP adds a granularity layer inside a wallet.
///
/// Using BIP32, it is possible to derive new deterministic key pairs using a derivation path, creating kind of subwallets called accounts.
/// Each accounts has it own extended private key, allowing them to spend bitcoins received on addresses generated with its associated extended public key, but preventing them from spending other wallet's accounts coins.
///
/// This feature can be useful for privacy purpose (see Samourai usage of accounts) or for businesses that want to separate revenue channels, but this is mostly useful to avoid user complaints from not finding their accounts previously on other wallet providers.
/// From a technical perspective, the code might be confusing as BDK use the "wallet" naming for whatever interacts with private keys, either master ones (wallet) or derived ones (accounts). Thus, in the codebase you might see this kind of interaction: A bitcoin Wallet generated from mnemonic, derived into an Account that instantiates a BDK Wallet.
#[derive(Debug)]
pub struct Account<Storage>
where
    Storage: BatchDatabase,
{
    derivation_path: DerivationPath,
    wallet: Wallet<Storage>,
}

#[derive(Clone, Copy, Debug)]
pub enum ScriptType {
    /// Legacy scripts : https://bitcoinwiki.org/wiki/pay-to-pubkey-hash
    Legacy,
    /// Nested segwit scrips : https://bitcoinwiki.org/wiki/pay-to-script-hash
    NestedSegwit,
    /// Native segwit scripts : https://bips.dev/173/
    NativeSegwit,
    /// Taproot scripts : https://bips.dev/341/
    Taproot,
}

impl TryFrom<String> for ScriptType {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == "legacy" {
            return Ok(ScriptType::Legacy);
        } else if value == "nested_segwit" {
            return Ok(ScriptType::NestedSegwit);
        } else if value == "native_segwit" {
            return Ok(ScriptType::NativeSegwit);
        } else if value == "taproot" {
            return Ok(ScriptType::Taproot);
        }

        Err(Error::InvalidScriptType)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AccountConfig {
    pub script_type: ScriptType,
    pub network: Network,
    pub account_index: u32,
    pub multisig_threshold: Option<(u32, u32)>,
}

impl AccountConfig {
    pub fn new(
        script_type: ScriptType,
        network: Network,
        account_index: u32,
        multisig_threshold: Option<(u32, u32)>,
    ) -> Self {
        Self {
            script_type,
            network,
            account_index,
            multisig_threshold,
        }
    }
}

pub fn build_account_derivation_path(config: AccountConfig) -> Result<Vec<ChildNumber>, Error> {
    let purpose = ChildNumber::from_hardened_idx(if config.multisig_threshold.is_some() {
        // TODO maybe support legacy standard (45' + 48')
        87 // https://bips.dev/87/
    } else {
        match config.script_type {
            ScriptType::Legacy => 44,       // https://bips.dev/44/
            ScriptType::NestedSegwit => 49, // https://bips.dev/49/
            ScriptType::NativeSegwit => 84, // https://bips.dev/84/
            ScriptType::Taproot => 86,      // https://bips.dev/86/
        }
    })
    .unwrap_or_else(|_| unreachable!("an error occured while generating child number from bip"));

    let coin_type = ChildNumber::from_hardened_idx(match config.network {
        Network::Bitcoin => 0,
        _ => 1,
    })
    .unwrap();

    let account = ChildNumber::from_hardened_idx(config.account_index).map_err(|_| Error::InvalidAccountIndex)?;

    Ok(vec![purpose, coin_type, account])
}

type ReturnedDescriptor = (
    Descriptor<DescriptorPublicKey>,
    HashMap<DescriptorPublicKey, DescriptorSecretKey>,
    HashSet<BdkNetwork>,
);

fn build_account_descriptors(
    account_xprv: ExtendedPrivKey,
    config: AccountConfig,
) -> Result<(ReturnedDescriptor, ReturnedDescriptor), Error> {
    let builder = if config.multisig_threshold.is_some() {
        todo!()
    } else {
        match config.script_type {
            ScriptType::Legacy => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(pkh(xkey)),
            ScriptType::NestedSegwit => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(sh(wpkh(xkey))),
            ScriptType::NativeSegwit => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(wpkh(xkey)),
            ScriptType::Taproot => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(tr(xkey)),
        }
    };

    let internal = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::Internal as u32,
        }]
        .into(),
    ))
    .map_err(|e| e.into())?;

    let external = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::External as u32,
        }]
        .into(),
    ))
    .map_err(|e| e.into())?;

    Ok((external, internal))
}

impl<Storage> Account<Storage>
where
    Storage: BatchDatabase,
{
    fn build_wallet(
        account_xprv: ExtendedPrivKey,
        config: AccountConfig,
        storage: Storage,
    ) -> Result<Wallet<Storage>, Error> {
        let (external_descriptor, internal_descriptor) = build_account_descriptors(account_xprv, config)?;

        Wallet::new(
            external_descriptor,
            Some(internal_descriptor),
            config.network.into(),
            storage,
        )
        .map_err(|e| e.into())
    }

    /// Returns a mutable reference to account's BdkWallet struct
    pub fn get_mutable_wallet(&mut self) -> &mut Wallet<Storage> {
        &mut self.wallet
    }

    /// Returns a reference to account's BdkWallet struct
    pub fn get_wallet(&self) -> &Wallet<Storage> {
        &self.wallet
    }

    /// From a master private key, returns a bitcoin account (as defined in https://bips.dev/44/)
    ///
    /// # Arguments
    ///
    /// * master_secret_key : the master private key of the wallet
    /// * config : config of the account, including script_type, network and index
    /// * storage : storage to persist account wallet data
    ///
    /// ```rust
    /// # use bdk::bitcoin::bip32::ExtendedPrivKey;
    /// # use bdk::database::MemoryDatabase;
    ///
    /// # use andromeda_bitcoin::account::{Account, AccountConfig, ScriptType};
    /// # use andromeda_bitcoin::mnemonic::Mnemonic;
    /// # use andromeda_bitcoin::bitcoin::Network;
    ///
    /// let mnemonic = Mnemonic::from_string(String::from("desk prevent enhance husband hungry idle member vessel room moment simple behave")).unwrap();
    /// let mprv = ExtendedPrivKey::new_master(Network::Testnet.into(), &mnemonic.inner().to_seed("")).unwrap();
    /// let config = AccountConfig::new(ScriptType::NativeSegwit, Network::Testnet, 0, None);
    /// let account = Account::new(mprv, config, MemoryDatabase::new());
    /// ```
    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig, storage: Storage) -> Result<Self, Error> {
        let secp = Secp256k1::new();

        let derivation_path = build_account_derivation_path(config)?;
        let account_xprv = master_secret_key
            .derive_priv(&secp, &derivation_path)
            .map_err(|e| e.into())?;

        Ok(Self {
            derivation_path: derivation_path.into(),
            wallet: Self::build_wallet(account_xprv, config, storage)?,
        })
    }

    /// Returns cloned derivation path
    pub fn get_derivation_path(&self) -> DerivationPath {
        self.derivation_path.clone()
    }

    /// Returns the last synced balance of an account.
    ///
    /// # Notes
    ///
    /// Balance details includes :
    /// * immature coins
    /// * trusted pending (unconfirmed internal)
    /// * untrusted pending (unconfirmed external)
    /// * confirmed coins
    pub fn get_balance(&self) -> Result<BdkBalance, Error> {
        self.wallet.get_balance().map_err(|e| e.into())
    }

    /// Returns a list of unspent outputs as a vector
    ///
    /// # Notes
    ///
    /// Later we might want to add pagination on top of that.
    pub fn get_utxos(&self) -> Result<Vec<LocalUtxo>, Error> {
        self.wallet.list_unspent().map_err(|e| e.into())
    }

    /// From a master private key, returns a bitcoin account (as defined in https://bips.dev/44/)
    ///
    /// # Note
    ///
    /// If index is None, it will return last unused address of the account. So to avoid address reuse, we need to sync before calling this method.
    pub fn get_address(&mut self, index: Option<u32>) -> Result<AddressInfo, Error> {
        let index = index.map_or(AddressIndex::LastUnused, |index| AddressIndex::Peek(index));
        self.wallet.get_address(index).map_err(|e| e.into())
    }

    /// Returns a boolean indicating whether or not the account owns the provided address
    pub fn owns(&self, address: &Address) -> Result<bool, Error> {
        self.wallet.is_mine(&address.script_pubkey()).map_err(|e| e.into())
    }

    /// Returns a bitcoin uri as defined in https://bips.dev/21/
    pub fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<PaymentLink, Error> {
        PaymentLink::new_bitcoin_uri(self, index, amount, label, message)
    }

    /// Returns a list of transactions, optionnally paginated. Maybe later we might force the pagination if not provided.
    ///
    /// # Notes
    ///
    /// Returned transaction are simple ones with only amount value, txid, confirmation time and fees value. For more details, `get_transaction` can be called with txid
    pub fn get_transactions(
        &self,
        pagination: Option<Pagination>,
        sorted: bool,
    ) -> Result<Vec<SimpleTransaction>, Error> {
        let pagination = pagination.unwrap_or_default();

        // We first need to sort transactions by their time (last_seen for unconfirmed ones and confirmation_time for confirmed one)
        // The collection that happen here might be consuming, maybe later we need to rework this part
        let simple_txs = self
            .wallet
            .list_transactions(true)
            .map_err(|e| e.into())?
            .into_iter()
            .map(|tx| SimpleTransaction::from_detailled_tx(tx, Some(self.derivation_path.clone())))
            .collect::<Vec<_>>();

        Ok(sort_and_paginate_txs(simple_txs, pagination, sorted))
    }

    /// Given a txid, returns a complete transaction    
    pub fn get_transaction(&self, txid: String) -> Result<TransactionDetails, Error> {
        let txid = Txid::from_str(&txid).map_err(|_| Error::InvalidTxId)?;

        let tx = self
            .wallet
            .get_tx(&txid, false)
            .map_err(|e| e.into())?
            .ok_or(Error::TransactionNotFound)?;

        TransactionDetails::from_bdk(tx, self.get_wallet())
    }

    /// Given a mutable reference to a PSBT, and sign options, tries to sign inputs elligible
    pub fn sign(
        &self,
        psbt: &mut PartiallySignedTransaction,
        sign_options: Option<SignOptions>,
    ) -> Result<bool, Error> {
        let sign_options = sign_options.unwrap_or_default();

        self.wallet.sign(psbt, sign_options).map_err(|e| e.into())
    }

    /// Broadcasts a given transaction
    pub async fn broadcast(&self, transaction: Transaction) -> Result<(), Error> {
        let blockchain = EsploraBlockchain::new("https://mempool.space/testnet/api", 20);

        blockchain
            .broadcast(&transaction)
            .await
            .map_err(|_| Error::CannotBroadcastTransaction)
    }

    /// Perform a full sync for the account
    pub async fn full_sync(&self) -> Result<(), Error> {
        let blockchain = EsploraBlockchain::new("https://mempool.space/testnet/api", 20);

        self.wallet
            .sync(&blockchain, SyncOptions::default())
            .await
            .map_err(|e| e.into())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bdk::database::MemoryDatabase;
    use miniscript::bitcoin::{bip32::ExtendedPrivKey, Address};

    use crate::bitcoin::Network;
    use crate::mnemonic::Mnemonic;

    use super::{Account, AccountConfig, ScriptType};

    fn set_test_account(script_type: ScriptType) -> Account<MemoryDatabase> {
        let config = AccountConfig::new(script_type, Network::Testnet, 0, None);

        let mnemonic = Mnemonic::from_string("category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade".to_string()).unwrap();
        let master_secret_key =
            ExtendedPrivKey::new_master(config.network.into(), &mnemonic.inner().to_seed("")).unwrap();

        Account::new(master_secret_key, config, MemoryDatabase::new()).unwrap()
    }

    #[test]
    fn get_correct_derivation_legacy() {
        let account = set_test_account(ScriptType::Legacy);
        assert_eq!(account.get_derivation_path().to_string(), "m/44'/1'/0'".to_string());
    }

    #[test]
    fn get_correct_derivation_native_segwit() {
        let account = set_test_account(ScriptType::NativeSegwit);
        assert_eq!(account.get_derivation_path().to_string(), "m/84'/1'/0'".to_string());
    }

    #[test]
    fn get_correct_derivation_nested_segwit() {
        let account = set_test_account(ScriptType::NestedSegwit);
        assert_eq!(account.get_derivation_path().to_string(), "m/49'/1'/0'".to_string());
    }

    #[test]
    fn get_correct_derivation_taproot() {
        let account = set_test_account(ScriptType::Taproot);
        assert_eq!(account.get_derivation_path().to_string(), "m/86'/1'/0'".to_string());
    }

    #[test]
    fn get_address_by_index_legacy() {
        let mut account = set_test_account(ScriptType::Legacy);
        assert_eq!(
            account.get_address(Some(13)).unwrap().to_string(),
            "mvqqkX5UmaqPvzS4Aa1gMhj4NFntGmju2N".to_string()
        );
    }

    #[test]
    fn get_address_by_index_nested_segwit() {
        let mut account = set_test_account(ScriptType::NestedSegwit);
        assert_eq!(
            account.get_address(Some(13)).unwrap().to_string(),
            "2MzYfE5Bt1g2A9zDBocPtcDjRqpFfdCeqe3".to_string()
        );
    }

    #[test]
    fn get_address_by_index_native_segwit() {
        let mut account = set_test_account(ScriptType::NativeSegwit);
        assert_eq!(
            account.get_address(Some(13)).unwrap().to_string(),
            "tb1qre68v280t3t5mdy0hcu86fnx3h289h0arfe6lr".to_string()
        );
    }

    #[test]
    fn get_address_by_index_taproot() {
        let mut account = set_test_account(ScriptType::Taproot);
        assert_eq!(
            account.get_address(Some(13)).unwrap().to_string(),
            "tb1ppanhpmq38z6738s0mwnd9h0z2j5jv7q4x4pc2wxqu8jw0gwmf69qx3zpaf".to_string()
        );
    }

    #[test]
    fn get_last_unused_address() {
        let mut account = set_test_account(ScriptType::Taproot);
        assert_eq!(
            account.get_address(None).unwrap().to_string(),
            "tb1pvv0tcny86mz4lsx97p03fvkkc09cg5nx5nvnxc7c323jv5sr6wnshfu377".to_string()
        );
    }

    #[test]
    fn get_bitcoin_uri_with_params() {
        let mut account = set_test_account(ScriptType::NativeSegwit);
        assert_eq!(
            account
                .get_bitcoin_uri(Some(5), Some(788927), Some("Hello world".to_string()), None)
                .unwrap()
                .to_string(),
            "bitcoin:tb1qkwfhq25jnjq4fca2tptdhpsstz9ss2pampswhc?amount=0.00788927&label=Hello%20world".to_string()
        );
    }

    #[test]
    fn get_is_address_owned_by_account() {
        let mut account = set_test_account(ScriptType::Taproot);

        let address = account.get_address(None).unwrap();
        assert!(account.owns(&address).unwrap());

        assert_eq!(
            account
                .owns(
                    &Address::from_str("tb1qkwfhq25jnjq4fca2tptdhpsstz9ss2pampswhc")
                        .unwrap()
                        .assume_checked()
                )
                .unwrap(),
            false
        );
    }
}
