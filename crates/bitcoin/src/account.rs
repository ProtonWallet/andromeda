use std::{collections::BTreeMap, fmt::Debug, str::FromStr, sync::Arc};

use andromeda_common::{utils::now, Network, ScriptType};
use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::{
    bitcoin::{
        bip32::{ChildNumber, DerivationPath, Xpriv},
        constants::genesis_block,
        psbt::Psbt,
        secp256k1::Secp256k1,
        Address, Network as BdkNetwork, Transaction, Txid,
    },
    chain::ConfirmationTime,
    descriptor,
    wallet::{AddressInfo, Balance as BdkBalance, Update},
    KeychainKind, LocalOutput as LocalUtxo, SignOptions, Wallet as BdkWallet,
};
use bitcoin::params::Params;
use miniscript::{descriptor::DescriptorSecretKey, DescriptorPublicKey};

use super::{payment_link::PaymentLink, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    error::Error,
    storage::{WalletStore, WalletStoreFactory},
    transactions::{ToTransactionDetails, TransactionDetails},
    utils::SortOrder,
};

const EXTERNAL_KEYCHAIN: KeychainKind = KeychainKind::External;

/// TLDR; A wallet is defined by its mnemonic + passphrase combo whereas a
/// wallet account is defined by its derivation path from the wallet masterkey.
/// In order to support wallet import from other major softwares, it has been
/// decided to support the BIP44 standard from the very beginning. This BIP adds
/// a granularity layer inside a wallet.
///
/// Using BIP32, it is possible to derive new deterministic key pairs using a
/// derivation path, creating kind of subwallets called accounts. Each accounts
/// has it own extended private key, allowing them to spend bitcoins received on
/// addresses generated with its associated extended public key, but preventing
/// them from spending other wallet's accounts coins.
///
/// This feature can be useful for privacy purpose (see Samourai usage of
/// accounts) or for businesses that want to separate revenue channels, but this
/// is mostly useful to avoid user complaints from not finding their accounts
/// previously on other wallet providers. From a technical perspective, the code
/// might be confusing as BDK use the "wallet" naming for whatever interacts
/// with private keys, either master ones (wallet) or derived ones (accounts).
/// Thus, in the codebase you might see this kind of interaction: A bitcoin
/// Wallet generated from mnemonic, derived into an Account that instantiates a
/// BDK Wallet.
#[derive(Debug, Clone)]
pub struct Account<P: WalletStore = ()> {
    derivation_path: DerivationPath,
    store: P,
    wallet: Arc<RwLock<BdkWallet>>,
}

type ReturnedDescriptor = (
    miniscript::Descriptor<DescriptorPublicKey>,
    BTreeMap<DescriptorPublicKey, DescriptorSecretKey>,
    std::collections::HashSet<BdkNetwork>,
);

fn build_account_descriptors(
    account_xprv: Xpriv,
    script_type: ScriptType,
) -> Result<(ReturnedDescriptor, ReturnedDescriptor), Error> {
    let builder = match script_type {
        ScriptType::Legacy => |xkey: (Xpriv, DerivationPath)| descriptor!(pkh(xkey)),
        ScriptType::NestedSegwit => |xkey: (Xpriv, DerivationPath)| descriptor!(sh(wpkh(xkey))),
        ScriptType::NativeSegwit => |xkey: (Xpriv, DerivationPath)| descriptor!(wpkh(xkey)),
        ScriptType::Taproot => |xkey: (Xpriv, DerivationPath)| descriptor!(tr(xkey)),
    };

    let internal = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::Internal as u32,
        }]
        .into(),
    ))?;

    let external = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::External as u32,
        }]
        .into(),
    ))?;

    Ok((external, internal))
}

impl<P: WalletStore> Account<P> {
    fn build_wallet_with_descriptors(
        external_descriptor: ReturnedDescriptor,
        internal_descriptor: ReturnedDescriptor,
        network: Network,
        store: P,
    ) -> Result<BdkWallet, Error> {
        let store_content = store.read()?;

        let genesis_block_hash = genesis_block(Params::from(&network.into())).block_hash();

        let wallet = BdkWallet::new_or_load_with_genesis_hash(
            external_descriptor,
            internal_descriptor,
            store_content,
            network.into(),
            genesis_block_hash,
        )?;

        Ok::<BdkWallet, Error>(wallet)
    }

    fn build_wallet(
        account_xprv: Xpriv,
        network: Network,
        script_type: ScriptType,
        store: P,
    ) -> Result<BdkWallet, Error> {
        let (external_descriptor, internal_descriptor) = build_account_descriptors(account_xprv, script_type)?;

        let wallet = Self::build_wallet_with_descriptors(
            external_descriptor.clone(),
            internal_descriptor.clone(),
            network,
            store.clone(),
        )
        .ok();

        if let Some(wallet) = wallet {
            return Ok(wallet);
        }

        store.clear()?;
        Self::build_wallet_with_descriptors(external_descriptor, internal_descriptor, network, store)
    }

    /// Returns a readable lock to account's BdkWallet struct
    pub async fn get_wallet(&self) -> RwLockReadGuard<BdkWallet> {
        self.wallet.read().await
    }

    /// Returns mutable lock a reference to account's BdkWallet struct
    pub async fn get_mutable_wallet(&self) -> RwLockWriteGuard<BdkWallet> {
        self.wallet.write().await
    }

    /// From a master private key, returns a bitcoin account (as defined in https://bips.dev/44/)
    ///
    /// # Arguments
    ///
    /// * master_secret_key : the master private key of the wallet
    /// * config : config of the account, including script_type, network and
    ///   index
    /// * store : store to persist account wallet data
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use bdk_wallet::bitcoin::{NetworkKind, bip32::{DerivationPath, Xpriv}};
    /// #
    /// # use andromeda_bitcoin::account::{Account};
    /// # use andromeda_bitcoin::mnemonic::Mnemonic;
    /// # use andromeda_common::{Network, ScriptType};
    /// # tokio_test::block_on(async {
    /// #
    /// let mnemonic = Mnemonic::from_string(String::from("desk prevent enhance husband hungry idle member vessel room moment simple behave")).unwrap();
    /// let mprv = Xpriv::new_master(NetworkKind::Test, &mnemonic.inner().to_seed("")).unwrap();
    /// let account = Account::new(mprv, Network::Testnet, ScriptType::NativeSegwit, DerivationPath::from_str("m/86'/1'/0'").unwrap(), ());
    /// # })
    /// ```
    pub fn new<F>(
        master_secret_key: Xpriv,
        network: Network,
        script_type: ScriptType,
        derivation_path: DerivationPath,
        factory: F,
    ) -> Result<Self, Error>
    where
        F: WalletStoreFactory<P>,
    {
        let secp = Secp256k1::new();

        let account_xprv = master_secret_key.derive_priv(&secp, &derivation_path)?;

        let store_key = format!("{}_{}", master_secret_key.fingerprint(&secp), derivation_path);
        let store = factory.build(store_key);

        Ok(Self {
            derivation_path,
            store: store.clone(),
            wallet: Arc::new(RwLock::new(Self::build_wallet(
                account_xprv,
                network,
                script_type,
                store,
            )?)),
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
    pub async fn get_balance(&self) -> BdkBalance {
        self.get_wallet().await.balance()
    }

    /// Returns a list of unspent outputs as a vector
    ///
    /// # Notes
    ///
    /// Later we might want to add pagination on top of that.
    pub async fn get_utxos(&self) -> Vec<LocalUtxo> {
        self.get_wallet().await.list_unspent().collect::<Vec<_>>()
    }

    /// Returns a bitcoin address (as defined in https://bips.dev/44/)
    ///
    /// # Note
    ///
    /// If index is None, it will return last unused address of the account. So
    /// to avoid address reuse, we need to sync before calling this method.
    ///
    /// Additionally, it takes care of revealing addresses up to the peeked
    /// index (or last unused address) and store the update
    pub async fn get_address(&self, index: Option<u32>) -> Result<AddressInfo, Error> {
        let mut write_lock = self.get_mutable_wallet().await;

        let address = if let Some(index) = index {
            // so far, there is no use-case to get internal keychain address
            write_lock.peek_address(EXTERNAL_KEYCHAIN, index)
        } else {
            write_lock.next_unused_address(EXTERNAL_KEYCHAIN)
        };

        // Here we only want to make sure we revealed addresses up to the peeked index
        // so that we detect transactions on partial syncings
        let _ = write_lock.reveal_addresses_to(EXTERNAL_KEYCHAIN, address.index);

        // We make sure the newly revealed addresses are stored
        self.store_stage(&mut write_lock)?;

        Ok(address)
    }

    /// Returns the last unused index from account
    ///
    /// # Note
    ///
    /// You need to take care of syncing the account before call this
    #[deprecated(
        note = "this fn returns next unused spk after of last unused. please use `get_index_after_last_used_address` instead"
    )]
    pub async fn get_last_unused_address_index(&self) -> Option<u32> {
        // so far, there is no use-case to get internal keychain address

        let wallet_lock = self.get_wallet();
        let mut spks = wallet_lock.await.spk_index().clone();

        spks.next_unused_spk(&EXTERNAL_KEYCHAIN).map(|spk| {
            let ((index, _), _) = spk;
            index
        })
    }

    /// Returns the index directly after last used address
    ///
    /// # Note
    ///
    /// You need to take care of syncing the account before call this
    pub async fn get_index_after_last_used_address(&self) -> u32 {
        // so far, there is no use-case to get internal keychain address

        let wallet_lock = self.get_wallet();
        let spks = wallet_lock.await.spk_index().clone();

        spks.last_used_index(&EXTERNAL_KEYCHAIN)
            .map(|index| index + 1)
            .unwrap_or(0)
    }

    /// Returns a boolean indicating whether or not the account owns the
    /// provided address
    pub async fn owns(&self, address: &Address) -> bool {
        self.get_wallet().await.is_mine(&address.script_pubkey())
    }

    /// Returns a bitcoin uri as defined in https://bips.dev/21/
    pub async fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<PaymentLink, Error> {
        PaymentLink::new_bitcoin_uri(self, index, amount, label, message).await
    }

    /// Returns a list of transactions, optionnally paginated. Maybe later we
    /// might force the pagination if not provided.
    ///
    /// # Notes
    ///
    /// Returned transaction are simple ones with only amount value, txid,
    /// confirmation time and fees value. For more details, `get_transaction`
    /// can be called with txid
    pub async fn get_transactions(
        &self,
        pagination: Option<Pagination>,
        sort: Option<SortOrder>,
    ) -> Result<Vec<TransactionDetails>, Error> {
        let pagination = pagination.unwrap_or_default();

        let wallet_lock = self.get_wallet().await;
        let transactions = wallet_lock.transactions().collect::<Vec<_>>();

        // We first need to sort transactions by their time (last_seen for unconfirmed
        // ones and confirmation_time for confirmed one) The collection that
        // happen here might be consuming, maybe later we need to rework this part
        let transactions = transactions
            .into_iter()
            .map(|tx| tx.to_transaction_details((&wallet_lock, (self.get_derivation_path()))))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sort_and_paginate_txs(transactions, pagination, sort))
    }

    /// Given a txid, returns a complete transaction    
    pub async fn get_transaction(&self, txid: String) -> Result<TransactionDetails, Error> {
        let txid = Txid::from_str(&txid)?;

        let wallet_lock = self.get_wallet().await;
        let tx = wallet_lock
            .transactions()
            .find(|tx| tx.tx_node.compute_txid() == txid)
            .ok_or(Error::TransactionNotFound)?;

        tx.to_transaction_details((&wallet_lock, self.get_derivation_path()))
    }

    /// Given a mutable reference to a PSBT, and sign options, tries to sign
    /// inputs elligible
    pub async fn sign(&self, psbt: &mut Psbt, sign_options: Option<SignOptions>) -> Result<(), Error> {
        let sign_options = sign_options.unwrap_or_default();
        self.get_wallet().await.sign(psbt, sign_options)?;

        Ok(())
    }

    /// Returns whether or not the account's wallet has already been synced at
    /// least once
    pub async fn has_sync_data(&self) -> bool {
        let wallet_lock = self.get_wallet().await;
        wallet_lock.latest_checkpoint().hash() != genesis_block(wallet_lock.network()).block_hash()
    }

    /// Manually insert a transaction not confirmed yet
    /// It will be then be stored and synced until it gets confirmed
    pub async fn insert_unconfirmed_tx(&self, tx: Transaction) -> Result<(), Error> {
        let mut wallet_lock = self.get_mutable_wallet().await;
        wallet_lock.insert_tx(
            tx,
            ConfirmationTime::Unconfirmed {
                last_seen: now().as_secs(),
            },
        )?;

        self.store_stage(&mut wallet_lock)?;

        Ok(())
    }

    pub async fn apply_update(&self, update: impl Into<Update>) -> Result<(), Error> {
        let mut wallet_lock = self.get_mutable_wallet().await;
        wallet_lock.apply_update(update)?;

        self.store_stage(&mut wallet_lock)?;

        Ok(())
    }

    pub fn store_stage(&self, wallet_lock: &mut RwLockWriteGuard<'_, BdkWallet>) -> Result<(), Error> {
        if let Some(changeset) = wallet_lock.take_staged() {
            self.store.write(&changeset)?;
        }

        Ok(())
    }

    pub fn clear_store(&self) -> Result<(), Error> {
        self.store.clear()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use andromeda_common::Network;
    use bdk_wallet::bitcoin::{
        bip32::{DerivationPath, Xpriv},
        Address, NetworkKind,
    };

    use super::{Account, ScriptType};
    use crate::mnemonic::Mnemonic;

    fn set_test_account(script_type: ScriptType, derivation_path: &str) -> Account<()> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string("category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade".to_string()).unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(master_secret_key, Network::Testnet, script_type, derivation_path, ()).unwrap()
    }

    #[tokio::test]
    async fn get_address_by_index_legacy() {
        let account = set_test_account(ScriptType::Legacy, "m/44'/1'/0'");
        assert_eq!(
            account.get_address(Some(13)).await.unwrap().to_string(),
            "mvqqkX5UmaqPvzS4Aa1gMhj4NFntGmju2N".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_nested_segwit() {
        let account = set_test_account(ScriptType::NestedSegwit, "m/49'/1'/0'");
        assert_eq!(
            account.get_address(Some(13)).await.unwrap().to_string(),
            "2MzYfE5Bt1g2A9zDBocPtcDjRqpFfdCeqe3".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_native_segwit() {
        let account = set_test_account(ScriptType::NativeSegwit, "m/84'/1'/0'");
        assert_eq!(
            account.get_address(Some(13)).await.unwrap().to_string(),
            "tb1qre68v280t3t5mdy0hcu86fnx3h289h0arfe6lr".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_taproot() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");
        assert_eq!(
            account.get_address(Some(13)).await.unwrap().to_string(),
            "tb1ppanhpmq38z6738s0mwnd9h0z2j5jv7q4x4pc2wxqu8jw0gwmf69qx3zpaf".to_string()
        );
    }

    #[tokio::test]
    async fn get_last_unused_address() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");
        assert_eq!(
            account.get_address(None).await.unwrap().to_string(),
            "tb1pvv0tcny86mz4lsx97p03fvkkc09cg5nx5nvnxc7c323jv5sr6wnshfu377".to_string()
        );
    }

    #[tokio::test]
    async fn get_bitcoin_uri_with_params() {
        let mut account = set_test_account(ScriptType::NativeSegwit, "m/84'/1'/0'");
        assert_eq!(
            account
                .get_bitcoin_uri(Some(5), Some(788927), Some("Hello world".to_string()), None)
                .await
                .unwrap()
                .to_string(),
            "bitcoin:tb1qkwfhq25jnjq4fca2tptdhpsstz9ss2pampswhc?amount=0.00788927&label=Hello%20world".to_string()
        );
    }

    #[tokio::test]
    async fn get_is_address_owned_by_account() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");

        let address = account.get_address(None).await.unwrap();
        assert!(account.owns(&address).await);

        assert!(
            !account
                .owns(
                    &Address::from_str("tb1qkwfhq25jnjq4fca2tptdhpsstz9ss2pampswhc")
                        .unwrap()
                        .assume_checked()
                )
                .await
        );
    }
}
