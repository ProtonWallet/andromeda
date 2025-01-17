use std::{collections::BTreeMap, fmt::Debug, str::FromStr, sync::Arc};

use andromeda_common::{utils::now, Network, ScriptType};
use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::{
    bitcoin::{
        bip32::{ChildNumber, DerivationPath, Xpriv},
        constants::genesis_block,
        psbt::Psbt as BdkPsbt,
        secp256k1::Secp256k1,
        Address, Network as BdkNetwork, Transaction, Txid,
    },
    descriptor, AddressInfo, Balance as BdkBalance, ChangeSet, KeychainKind, LocalOutput as LocalUtxo, PersistedWallet,
    SignOptions, Update, Wallet as BdkWallet, WalletPersister,
};
use bitcoin::{params::Params, Amount};
use miniscript::{descriptor::DescriptorSecretKey, DescriptorPublicKey};

use super::{payment_link::PaymentLink, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    address::AddressDetails,
    bdk_wallet_ext::BdkWalletExt,
    blockchain_client::BlockchainClient,
    error::Error,
    psbt::Psbt,
    storage::{WalletConnectorFactory, WalletPersisterConnector},
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
pub struct Account<C: WalletPersisterConnector<P>, P: WalletPersister> {
    derivation_path: DerivationPath,
    wallet: Arc<RwLock<PersistedWallet<P>>>,
    persister_connector: C,
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

impl<C: WalletPersisterConnector<P>, P: WalletPersister> Account<C, P> {
    fn build_wallet_with_descriptors(
        external_descriptor: ReturnedDescriptor,
        internal_descriptor: ReturnedDescriptor,
        network: Network,
        persister: &mut P,
    ) -> Result<PersistedWallet<P>, Error>
    where
        C: WalletPersisterConnector<P>,
        P: WalletPersister,
    {
        let genesis_block_hash = genesis_block(Params::from(&network.into())).block_hash();

        let wallet_opt = BdkWallet::load()
            .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
            .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
            .extract_keys()
            .check_network(network.into())
            .check_genesis_hash(genesis_block_hash)
            .load_wallet(persister)
            // If we have an error loading wallet, we just create a new one
            .ok()
            .flatten();

        let wallet = match wallet_opt {
            Some(wallet) => wallet,
            None => BdkWallet::create(external_descriptor, internal_descriptor)
                .network(network.into())
                .genesis_hash(genesis_block_hash)
                .create_wallet(persister)
                .map_err(|_e| Error::CreateWithPersistError)?,
        };

        Ok(wallet)
    }

    fn build_wallet(
        account_xprv: Xpriv,
        network: Network,
        script_type: ScriptType,
        persister: &mut P,
    ) -> Result<PersistedWallet<P>, Error> {
        let (external_descriptor, internal_descriptor) = build_account_descriptors(account_xprv, script_type)?;

        let wallet = Self::build_wallet_with_descriptors(
            external_descriptor.clone(),
            internal_descriptor.clone(),
            network,
            persister,
        )
        .ok();

        if let Some(wallet) = wallet {
            return Ok(wallet);
        }

        Self::build_wallet_with_descriptors(external_descriptor, internal_descriptor, network, persister)
    }

    /// Returns a readable lock to account's BdkWallet struct
    pub async fn get_wallet(&self) -> RwLockReadGuard<PersistedWallet<P>> {
        self.wallet.read().await
    }

    /// Returns mutable lock a reference to account's BdkWallet struct
    pub async fn get_mutable_wallet(&self) -> RwLockWriteGuard<PersistedWallet<P>> {
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
    /// # use andromeda_bitcoin::storage::MemoryPersisted;
    /// # use andromeda_common::{Network, ScriptType};
    /// # tokio_test::block_on(async {
    /// #
    /// let mnemonic = Mnemonic::from_string(String::from("desk prevent enhance husband hungry idle member vessel room moment simple behave")).unwrap();
    /// let mprv = Xpriv::new_master(NetworkKind::Test, &mnemonic.inner().to_seed("")).unwrap();
    /// let account = Account::new(mprv, Network::Testnet, ScriptType::NativeSegwit, DerivationPath::from_str("m/86'/1'/0'").unwrap(), MemoryPersisted);
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
        F: WalletConnectorFactory<C, P>,
    {
        let secp = Secp256k1::new();

        let account_xprv = master_secret_key.derive_priv(&secp, &derivation_path)?;

        let store_key = format!("{}_{}", master_secret_key.fingerprint(&secp), derivation_path);

        let connector = factory.build(store_key);
        let mut persister = connector.connect();

        Ok(Self {
            derivation_path,
            persister_connector: connector.clone(),
            wallet: Arc::new(RwLock::new(Self::build_wallet(
                account_xprv,
                network,
                script_type,
                &mut persister,
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

    /// Marks a range of receive addresses (external keychain) as used and
    /// persists the changes.
    ///
    /// Acquires a mutable wallet lock, marks addresses from `from` to `to`
    /// (exclusive) as used, and persists the changes. If `to` is `None`, it
    /// defaults to `from + 1`.
    ///
    /// # Parameters
    /// - `from`: Starting index of addresses to mark as used.
    /// - `to`: Optional ending index (exclusive).
    pub async fn mark_receive_addresses_used_to(&self, from: u32, to: Option<u32>) -> Result<(), Error> {
        let mut write_lock = self.get_mutable_wallet().await;

        write_lock.mark_used_to(EXTERNAL_KEYCHAIN, from, to);

        Ok(())
    }

    /// Returns the highest used address index `Some(u32)` from the wallet's outputs for a specific keychain,
    /// or `None` if no outputs are found for the given keychain.
    /// # Parameters
    ///
    /// - `keychain`: The type of keychain (e.g., external or internal) to filter the outputs by.
    pub async fn get_highest_used_address_index_in_output(&self, keychain: KeychainKind) -> Result<Option<u32>, Error> {
        let highest_index = self
            .get_wallet()
            .await
            .list_output()
            .into_iter()
            .filter(|output| output.keychain == keychain)
            .map(|output| output.derivation_index)
            .reduce(|a, b| a.max(b));

        Ok(highest_index)
    }

    /// Returns the next address to be used to receive coins and marks it as
    /// used
    ///
    /// Note:
    /// The following should be done prior to calling this function:
    /// - Syncing chain data
    /// - Mark indexes up to LastUsedIndex (returned from API) as used
    /// - BvE pool bitcoin addresses must be marked as used
    pub async fn get_next_receive_address(&self) -> Result<AddressInfo, Error> {
        let mut write_lock = self.get_mutable_wallet().await;

        let address = write_lock.next_unused_address(EXTERNAL_KEYCHAIN);
        write_lock.mark_used(EXTERNAL_KEYCHAIN, address.index);

        Ok(address)
    }

    /// Peeks a specific address to be used to receive coins and marks it as
    /// used
    pub async fn peek_receive_address(&self, index: u32) -> Result<AddressInfo, Error> {
        let mut write_lock = self.get_mutable_wallet().await;

        let address = write_lock.peek_address(EXTERNAL_KEYCHAIN, index);
        write_lock.mark_used(EXTERNAL_KEYCHAIN, address.index);

        Ok(address)
    }

    /// Returns a boolean indicating whether or not the account owns the
    /// provided address
    pub async fn owns(&self, address: &Address) -> bool {
        self.get_wallet().await.is_mine(address.script_pubkey())
    }

    /// Returns the maximum gap size `Some(u32)` from the wallet's outputs for a specific keychain,
    /// or `None` if no outputs are found for the given keychain.
    /// # Parameters
    ///
    /// - `keychain`: The type of keychain (e.g., external or internal) to filter the outputs by.
    pub async fn get_maximum_gap_size(&self, keychain: KeychainKind) -> Result<Option<u32>, Error> {
        let mut used_indices: Vec<u32> = self
            .get_wallet()
            .await
            .list_output()
            .into_iter()
            .filter(|output| output.keychain == keychain)
            .map(|output| output.derivation_index)
            .collect();

        if used_indices.len() == 0 {
            // no index was used, return None
            return Ok(None);
        } else if used_indices.len() == 1 {
            // only 1 address was used, return the address index directly
            // for example, used_indices = [33]
            return Ok(Some(used_indices[0]));
        } else {
            // calculate the maximum gap between indices and return it
            // for example, used_indices = [33, 18, 100]
            used_indices.sort();
            // after sorted, used_indices = [18, 33, 100]

            // add 0 at first element, so we can get correct gap for first used index
            // used_indices will become [0, 18, 33, 100]
            used_indices.insert(0, 0);

            let max_gap = used_indices.windows(2).map(|indices| indices[1] - indices[0]).max();
            return Ok(max_gap);
        }
    }

    /// Returns a bitcoin uri as defined in https://bips.dev/21/
    pub async fn get_bitcoin_uri(
        &mut self,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<PaymentLink, Error> {
        let address = self.get_next_receive_address().await?;

        Ok(PaymentLink::new_bitcoin_uri(address.address, amount, label, message))
    }

    /// Returns a paginated list of transactions.
    ///
    /// # Notes
    ///
    /// Returned transaction are simple ones with only amount value, txid,
    /// confirmation time and fees value. For more details, `get_transaction`
    /// can be called with txid
    pub async fn get_transactions(
        &self,
        pagination: Pagination,
        sort: Option<SortOrder>,
    ) -> Result<Vec<TransactionDetails>, Error> {
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

    /// Returns a single address if found in the graph.
    ///
    /// # Notes
    ///
    /// The returned address detail contains balance and transactions
    /// with output to the address, in addition to index and serialised
    /// address. It can then be used to build an address list with enhanced
    /// details
    pub async fn get_address(
        &self,
        network: Network,
        address_str: String,
        client: Arc<BlockchainClient>,
        sync: bool,
    ) -> Result<Option<AddressDetails>, Error> {
        let spk: bitcoin::ScriptBuf = Address::from_str(&address_str)?
            .require_network(network.into())?
            .script_pubkey();

        let wallet_lock = self.get_wallet().await;

        if let Some((keychain, spk_index)) = wallet_lock.derivation_of_spk(spk.clone()) {
            let update = {
                if sync {
                    Some(client.sync_spks(&wallet_lock, vec![spk]).await?)
                } else {
                    None
                }
            };

            if let Some(update) = update {
                self.apply_update(update).await?;
            }

            let wallet_lock = self.get_wallet().await;

            let outpoints = wallet_lock.outpoints_from_spk_index(keychain, spk_index);
            let spk_balance = wallet_lock.tx_graph().balance(
                wallet_lock.local_chain(),
                wallet_lock.local_chain().tip().block_id(),
                outpoints,
                |_, _| false,
            );

            let transactions = wallet_lock
                .outpoints_from_spk_index(keychain, spk_index)
                .filter_map(|(_, op)| wallet_lock.tx_graph().get_tx_node(op.txid))
                .map(|tx_node| tx_node.to_transaction_details((&wallet_lock, self.get_derivation_path())))
                .collect::<Result<Vec<_>, _>>()?;

            let address_str = wallet_lock.peek_address(keychain, spk_index).address.to_string();

            return Ok(Some(AddressDetails {
                index: spk_index,
                address: address_str,
                balance: spk_balance,
                transactions,
            }));
        }

        Ok(None)
    }

    /// Returns a paginated list of addresses.
    ///
    /// # Notes
    ///
    /// Each returned address detail contains balance and transactions
    /// with output to the address, in addition to index and serialised
    /// address. It can then be used to build an address list with enhanced
    /// details
    pub async fn get_addresses(
        &self,
        pagination: Pagination,
        client: Arc<BlockchainClient>,
        keychain: KeychainKind,
        force_sync: bool,
    ) -> Result<Vec<AddressDetails>, Error> {
        let (skip, take) = (pagination.skip as u32, pagination.take as u32);
        let spks_range = skip..(skip + take - 1);

        // We need to reveal addresses to be able to sync them
        let mut wallet_lock = self.get_mutable_wallet().await;
        let _ = wallet_lock.reveal_addresses_to(keychain, spks_range.end);
        drop(wallet_lock);

        let update = {
            let wallet_lock = self.get_wallet().await;

            let spks = spks_range
                .clone()
                .filter_map(|index| wallet_lock.spk_index().spk_at_index(keychain, index))
                .collect::<Vec<_>>();

            // Sync missing spks (or all if forced)
            let spks_to_sync = if force_sync {
                spks
            } else {
                client.inner().filter_already_fetched(spks).await
            };

            if !spks_to_sync.is_empty() {
                Some(client.sync_spks(&wallet_lock, spks_to_sync).await?)
            } else {
                None
            }
        };

        if let Some(update) = update {
            self.apply_update(update).await?;
        }

        let wallet_lock = self.get_wallet().await;

        // Find tx data from spk
        let mut address_details = Vec::new();
        for spk_index in spks_range {
            let outpoints = wallet_lock.outpoints_from_spk_index(keychain, spk_index);
            let spk_balance = wallet_lock.tx_graph().balance(
                wallet_lock.local_chain(),
                wallet_lock.local_chain().tip().block_id(),
                outpoints,
                |_, _| false,
            );

            let transactions = wallet_lock
                .outpoints_from_spk_index(keychain, spk_index)
                .filter_map(|(_, op)| wallet_lock.tx_graph().get_tx_node(op.txid))
                .map(|tx_node| tx_node.to_transaction_details((&wallet_lock, self.get_derivation_path())))
                .collect::<Result<Vec<_>, _>>()?;

            let address_str = wallet_lock.peek_address(keychain, spk_index).address.to_string();

            address_details.push(AddressDetails {
                index: spk_index,
                address: address_str,
                balance: spk_balance,
                transactions,
            });
        }

        Ok(address_details)
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
    pub async fn sign(&self, psbt: &mut BdkPsbt, sign_options: Option<SignOptions>) -> Result<(), Error> {
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

    pub async fn bump_transactions_fees(&self, txid: String, fees: u64) -> Result<Psbt, Error> {
        let mut wallet_lock: RwLockWriteGuard<'_, PersistedWallet<P>> = self.get_mutable_wallet().await;
        let mut fee_bump_tx = wallet_lock.build_fee_bump(Txid::from_str(&txid)?)?;

        fee_bump_tx.fee_absolute(Amount::from_sat(fees));

        let psbt = fee_bump_tx.finish()?;

        Ok(psbt.into())
    }

    pub async fn apply_update(&self, update: impl Into<Update>) -> Result<(), Error> {
        let mut wallet_lock = self.get_mutable_wallet().await;
        wallet_lock.apply_update_at(update, now().as_secs())?;

        self.persist(wallet_lock).await?;

        Ok(())
    }

    async fn persist(&self, mut wallet_lock: RwLockWriteGuard<'_, PersistedWallet<P>>) -> Result<(), Error> {
        let mut persister = self.persister_connector.connect();

        wallet_lock.persist(&mut persister).map_err(|_e| Error::PersistError)?;
        drop(wallet_lock);

        Ok(())
    }

    pub fn clear_store(&self) -> Result<(), Error> {
        let mut persister = self.persister_connector.connect();

        P::persist(&mut persister, &ChangeSet::default()).map_err(|_e| Error::PersistError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc};

    use andromeda_api::{
        tests::utils::{common_api_client, setup_test_connection},
        BASE_WALLET_API_V1,
    };
    use andromeda_common::Network;
    use bdk_wallet::{
        bitcoin::{
            bip32::{DerivationPath, Xpriv},
            Address, NetworkKind,
        },
        serde_json,
    };
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{Account, ScriptType};
    use crate::{
        blockchain_client::BlockchainClient, mnemonic::Mnemonic, read_mock_file, storage::MemoryPersisted,
        transactions::Pagination, utils::SortOrder,
    };

    fn set_test_account(script_type: ScriptType, derivation_path: &str) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string("category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade".to_string()).unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Testnet,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    fn set_test_account_regtest(
        script_type: ScriptType,
        derivation_path: &str,
    ) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Regtest,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    fn set_test_account_regtest2(
        script_type: ScriptType,
        derivation_path: &str,
    ) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string(
            "remove over athlete patient priority unable memory axis sunset home balance sausage".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Regtest,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    fn set_test_account_regtest3(
        script_type: ScriptType,
        derivation_path: &str,
    ) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string(
            "submit exhibit banner enter situate exact north fog era family south style".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Regtest,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    fn set_test_account_for_mainnet(
        script_type: ScriptType,
        derivation_path: &str,
    ) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Main;
        let mnemonic = Mnemonic::from_string(
            "sense giggle bulb easily canal since toilet diagram deer able inflict bacon".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Bitcoin,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    async fn get_mock_server_for_mainnet() -> MockServer {
        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body_mainnet");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_mainnet_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5433431edc5b618c67ddc237ea86a996f32004022daa49bd40dc607052e892e0",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_mainnet_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "364923b99c5e330ddf19692077b8d69700e71083542ef0a6ebc9e657d5e1ef56",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_mainnet_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "032f964badcda0acf21c0b7642fd7a94056e3cfa49a38ed92d9c2f46acba4417",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents4 = read_mock_file!("get_scripthashes_transactions_body_mainnet_4");
        let response4 = ResponseTemplate::new(200).set_body_string(response_contents4);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "278a61a1f06888fab709301b570b9c3e4b9efdc54a1f26b9bfbad34b508e8433",
            ))
            .respond_with(response4)
            .mount(&mock_server)
            .await;

        let block_hash_data = vec![
            (0, "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"),
            (
                869089,
                "000000000000000000015c4d376f65d85378b3b6f5eaae656078dc319dd7fcdf",
            ),
            (
                869090,
                "000000000000000000002067d2cd097be40e1d7d1876d38b8025eb1bb2f28fc4",
            ),
            (
                869434,
                "00000000000000000000556ef62a310d7230ec72afc2a577b953072320f03c5b",
            ),
            (
                871611,
                "00000000000000000001f9a7c7c50359f190a71fcd658f5efe02808adfa8027b",
            ),
            (
                872365,
                "000000000000000000001d74687b6170827dad9bf59c4d612db3d7cd73009a5b",
            ),
            (
                872366,
                "000000000000000000018700dd168b5672808addc35d6aa015eb817e80a00f2d",
            ),
        ];
        for (block_height, block_hash) in block_hash_data {
            Mock::given(method("GET"))
                .and(path_regex(format!(".*blocks/height/{}/hash", block_height)))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "Code": 1000,
                    "BlockHash": block_hash,
                })))
                .mount(&mock_server)
                .await;
        }
        return mock_server;
    }

    #[tokio::test]
    async fn test_get_wallet() {
        let account = set_test_account(ScriptType::Legacy, "m/44'/1'/0'");
        let wallet = account.get_wallet().await;
        assert!(wallet.balance().total().to_sat() == 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_address_sync_true() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let network = Network::Regtest;
        let address_str = "bcrt1q4zpmdp77e9ff4ls8ajgqapdhgqutrkcpqpzcqw".to_string();

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());
        let address_detail = account
            .get_address(network, address_str.clone(), Arc::new(client.clone()), true)
            .await
            .unwrap();
        assert!(address_detail.is_some());
        assert!(address_detail.unwrap().balance.confirmed.to_sat() == 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_address_sync_false() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let network = Network::Regtest;
        let address_str = "bcrt1q4zpmdp77e9ff4ls8ajgqapdhgqutrkcpqpzcqw".to_string();

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());
        let address_detail = account
            .get_address(network, address_str.clone(), Arc::new(client.clone()), false)
            .await
            .unwrap();
        assert!(address_detail.is_some());
        assert!(address_detail.unwrap().balance.confirmed.to_sat() == 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_addresses_sync_false() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());
        let pagination = Pagination::new(0, 10);
        let addresses = account
            .get_addresses(
                pagination,
                Arc::new(client.clone()),
                bdk_wallet::KeychainKind::Internal,
                false,
            )
            .await
            .unwrap();
        assert!(addresses.len() == 9);
    }

    #[tokio::test]
    async fn test_get_addresses() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_addresses_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "3979339df938c20edd4f7505ba17ded9c999c4901091660072628aa0b69fe004",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());
        let pagination = Pagination::new(0, 10);
        let addresses = account
            .get_addresses(
                pagination,
                Arc::new(client.clone()),
                bdk_wallet::KeychainKind::Internal,
                false,
            )
            .await
            .unwrap();
        assert!(addresses.len() == 9);
    }

    #[tokio::test]
    async fn test_get_address() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let network = Network::Regtest;
        let address_str = "bcrt1q4zpmdp77e9ff4ls8ajgqapdhgqutrkcpqpzcqw".to_string();

        let mock_server = MockServer::start().await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());
        let address_detail = account
            .get_address(network, address_str.clone(), Arc::new(client.clone()), false)
            .await
            .unwrap();
        assert!(address_detail.is_some());
        assert!(address_detail.unwrap().balance.confirmed.to_sat() == 0);
    }

    #[tokio::test]
    async fn test_bump_transactions_fees_error() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let result = account
            .bump_transactions_fees(
                "6b62ad31e219c9dab4d7e24a0803b02bbc5d86ba53f6f02aa6de0f301b718e88".to_string(),
                1000,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_derivation_path() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let derivation_path = account.get_derivation_path();
        assert_eq!(derivation_path.to_string(), "84'/1'/0'");
    }

    #[tokio::test]
    async fn test_get_balance() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let balance = account.get_balance().await;
        assert_eq!(balance.total().to_sat(), 0);

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();
        let balance = account.get_balance().await;
        assert_eq!(balance.total().to_sat(), 8781);
    }

    #[tokio::test]
    async fn test_get_utxo() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");
        let utxos = account.get_utxos().await;
        assert_eq!(utxos.len(), 0);

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();
        let utxos = account.get_utxos().await;

        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].chain_position.is_confirmed(), true);
        assert_eq!(utxos[0].txout.value.to_sat(), 8781);
    }

    #[tokio::test]
    async fn test_bump_transactions_fees_success() {}

    #[tokio::test]
    async fn test_has_sync_data() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        let has_synced = account.has_sync_data().await;
        assert!(!has_synced);

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let has_synced = account.has_sync_data().await;
        assert!(has_synced);
    }

    #[tokio::test]
    async fn test_get_transactions() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());
        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        // get single transaction
        let txid = "6b62ad31e219c9dab4d7e24a0803b02bbc5d86ba53f6f02aa6de0f301b718e88".to_string();
        let transaction_details = account.get_transaction(txid).await.unwrap();
        assert_eq!(transaction_details.fees.unwrap(), 141);
        assert_eq!(transaction_details.received, 8781);

        // get transactions
        let pagination = Pagination::new(0, 10);
        let transactions = account
            .get_transactions(pagination, Some(SortOrder::Asc))
            .await
            .unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].fees.unwrap(), 141);
        assert_eq!(transactions[0].received, 8781);
    }

    #[tokio::test]
    async fn test_get_highest_used_address_index_in_output() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        // highest will be none before we sync the wallet
        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.is_none());

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert_eq!(highest.unwrap(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_highest_from_atlas() {
        let account = set_test_account_regtest2(ScriptType::NativeSegwit, "m/84'/1'/0'");

        // mark used to make sure it wont change the result of get_highest_used_address_index_in_output()
        account.mark_receive_addresses_used_to(0, Some(210)).await.unwrap();

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());

        // do full sync
        let update = client.full_sync(&account, Some(200)).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.unwrap() == 138);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_maximum_gap_size_from_atlas() {
        let account = set_test_account_regtest3(ScriptType::NativeSegwit, "m/84'/1'/0'");

        // mark used to make sure it wont change the result of get_highest_used_address_index_in_output()
        account.mark_receive_addresses_used_to(0, Some(210)).await.unwrap();

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());

        // do full sync
        let update = client.full_sync(&account, Some(500)).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let maximum_gap = account
            .get_maximum_gap_size(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(maximum_gap.unwrap() == 450);
    }

    #[tokio::test]
    async fn test_get_highest_used_address_index_in_output_mainnet() {
        let account = set_test_account_for_mainnet(ScriptType::NativeSegwit, "m/84'/0'/0'");
        // highest will be none before we sync the wallet
        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.is_none());

        // mark used to make sure it wont change the result of get_highest_used_address_index_in_output()
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        let mock_server = get_mock_server_for_mainnet().await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, Some(20)).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert_eq!(highest.unwrap(), 7);
    }

    #[tokio::test]
    async fn test_get_maximum_gap_size_mainnet() {
        let account = set_test_account_for_mainnet(ScriptType::NativeSegwit, "m/84'/0'/0'");
        // highest will be none before we sync the wallet
        let maximum_gap_size = account
            .get_maximum_gap_size(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(maximum_gap_size.is_none());

        // mark used to make sure it wont change the result of get_highest_used_address_index_in_output()
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        let mock_server = get_mock_server_for_mainnet().await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, Some(20)).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        // account has used [2, 4, 7] after full sync
        let maximum_gap_size = account
            .get_maximum_gap_size(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert_eq!(maximum_gap_size.unwrap(), 3);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_transactions_from_atlas() {
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let api_client = common_api_client().await;
        let client = BlockchainClient::new(api_client.as_ref().clone());

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        let txid = "6b62ad31e219c9dab4d7e24a0803b02bbc5d86ba53f6f02aa6de0f301b718e88".to_string();
        let transaction_details = account.get_transaction(txid).await.unwrap();
        assert_eq!(transaction_details.fees.unwrap(), 141);
    }

    #[tokio::test]
    async fn get_address_by_index_legacy() {
        let account = set_test_account(ScriptType::Legacy, "m/44'/1'/0'");
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "mvqqkX5UmaqPvzS4Aa1gMhj4NFntGmju2N".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_nested_segwit() {
        let account = set_test_account(ScriptType::NestedSegwit, "m/49'/1'/0'");
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "2MzYfE5Bt1g2A9zDBocPtcDjRqpFfdCeqe3".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_native_segwit() {
        let account = set_test_account(ScriptType::NativeSegwit, "m/84'/1'/0'");
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "tb1qre68v280t3t5mdy0hcu86fnx3h289h0arfe6lr".to_string()
        );

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "tb1qgpqu7j420k9yq2ua7q577unxd8lnw79er59tdx".to_string()
        );
    }

    #[tokio::test]
    async fn get_address_by_index_taproot() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");
        account.mark_receive_addresses_used_to(0, Some(13)).await.unwrap();

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "tb1ppanhpmq38z6738s0mwnd9h0z2j5jv7q4x4pc2wxqu8jw0gwmf69qx3zpaf".to_string()
        );
    }

    #[tokio::test]
    async fn get_last_unused_address() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");

        assert_eq!(
            account.get_next_receive_address().await.unwrap().to_string(),
            "tb1pvv0tcny86mz4lsx97p03fvkkc09cg5nx5nvnxc7c323jv5sr6wnshfu377".to_string()
        );
    }

    #[tokio::test]
    async fn get_bitcoin_uri_with_params() {
        let mut account = set_test_account(ScriptType::NativeSegwit, "m/84'/1'/0'");
        account.mark_receive_addresses_used_to(0, Some(5)).await.unwrap();

        assert_eq!(
            account
                .get_bitcoin_uri(Some(788927), Some("Hello world".to_string()), None)
                .await
                .unwrap()
                .to_string(),
            "bitcoin:tb1qkwfhq25jnjq4fca2tptdhpsstz9ss2pampswhc?amount=0.00788927&label=Hello%20world".to_string()
        );
    }

    #[tokio::test]
    async fn get_is_address_owned_by_account() {
        let account = set_test_account(ScriptType::Taproot, "m/86'/1'/0'");

        let address = account.get_next_receive_address().await.unwrap();
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
