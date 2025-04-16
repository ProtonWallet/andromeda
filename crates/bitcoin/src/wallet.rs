use core::fmt::Debug;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use andromeda_api::ProtonWalletApiClient;
use andromeda_common::{FromParts, Network, ScriptType};
use bdk_wallet::{
    bitcoin::{
        bip32::{DerivationPath, Xpriv},
        secp256k1::Secp256k1,
        Amount, NetworkKind,
    },
    Balance,
};
use bitcoin::bip32::Xpub;
use futures::future::try_join_all;

use super::{account::Account, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    account_syncer::AccountSyncer,
    account_trait::AccessWallet,
    blockchain_client::BlockchainClient,
    error::Error,
    mnemonic::Mnemonic,
    storage::{WalletPersisterFactory, WalletStorage},
    transactions::{ToTransactionDetails, TransactionDetails},
    utils::{SortOrder, TransactionFilter},
};

const ACCOUNT_DISCOVERY_STOP_GAP: u32 = 2;
const ADDRESS_DISCOVERY_STOP_GAP: usize = 10;

#[derive(Debug, PartialEq)]
pub enum WalletType {
    OnChain,
    Lightning,
    WatchOnly,
}

#[derive(Debug)]
enum WalletMasterKey {
    Xpriv(Xpriv),
    Xpub(Xpub),
}

#[derive(Debug)]
pub struct Wallet {
    key: WalletMasterKey,
    accounts: HashMap<DerivationPath, Arc<Account>>,
    network: Network,
}

impl Wallet {
    pub fn new(network: Network, bip39_mnemonic: String, bip38_passphrase: Option<String>) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_string(bip39_mnemonic)?;

        let network_kind = match network {
            Network::Bitcoin => NetworkKind::Main,
            _ => NetworkKind::Test,
        };

        let mprv = Xpriv::new_master(
            network_kind,
            &mnemonic.inner().to_seed(match bip38_passphrase {
                Some(bip38_passphrase) => bip38_passphrase,
                None => "".to_string(),
            }),
        )
        .unwrap();

        Ok(Wallet {
            key: WalletMasterKey::Xpriv(mprv),
            accounts: HashMap::new(),
            network,
        })
    }

    pub fn new_watch_only(network: Network, xpub: String) -> Result<Self, Error> {
        let xpub = Xpub::from_str(&xpub)?;

        Ok(Wallet {
            key: WalletMasterKey::Xpub(xpub),
            accounts: HashMap::new(),
            network,
        })
    }

    pub fn mprv(&self) -> (Option<Xpriv>, Network) {
        if let WalletMasterKey::Xpriv(mprv) = &self.key {
            return (Some(mprv.clone()), self.network);
        } else {
            return (None, self.network);
        }
    }

    pub fn xpub(&self) -> (Option<Xpub>, Network) {
        if let WalletMasterKey::Xpub(xpub) = &self.key {
            return (Some(xpub.clone()), self.network);
        } else {
            return (None, self.network);
        }
    }

    pub fn add_account<F>(
        &mut self,
        script_type: ScriptType,
        derivation_path: DerivationPath,
        factory: F,
    ) -> Result<Arc<Account>, Error>
    where
        F: WalletPersisterFactory,
    {
        let secp = Secp256k1::new();
        let account = if let WalletMasterKey::Xpriv(mprv) = self.key {
            let store_key = format!("{}_{}", mprv.fingerprint(&secp), derivation_path.to_string());
            let persister = factory.build(store_key);
            Account::new(
                mprv,
                self.network,
                script_type,
                derivation_path,
                WalletStorage(persister),
            )?
        } else if let WalletMasterKey::Xpub(xpub) = self.key {
            let store_key = format!("{}_{}", xpub.fingerprint(), derivation_path.to_string());
            let persister = factory.build(store_key);
            Account::new_with_xpub(
                xpub,
                script_type,
                self.network,
                derivation_path,
                WalletStorage(persister),
            )?
        } else {
            return Err(Error::WalletNotInitialized);
        };

        let derivation_path = account.get_derivation_path();

        let account_arc = Arc::new(account);

        self.accounts.insert(derivation_path.clone(), account_arc.clone());

        Ok(account_arc)
    }

    pub fn get_account(&self, derivation_path: &DerivationPath) -> Option<Arc<Account>> {
        self.accounts.get(derivation_path).cloned()
    }

    pub fn get_accounts(&self) -> Vec<Arc<Account>> {
        self.accounts.values().cloned().collect::<Vec<_>>()
    }

    pub async fn get_balance(&self) -> Result<Balance, Error> {
        let async_iter = self.accounts.keys().map(|account_key| async move {
            let account = self.accounts.get(account_key).ok_or(Error::AccountNotFound)?;

            Ok::<Balance, Error>(account.get_balance().await)
        });

        let account_balances = try_join_all(async_iter).await?;

        let init = Balance {
            untrusted_pending: Amount::from_sat(0),
            confirmed: Amount::from_sat(0),
            immature: Amount::from_sat(0),
            trusted_pending: Amount::from_sat(0),
        };

        let balance = account_balances
            .into_iter()
            .fold(init, |acc: Balance, account_balance| acc + account_balance);

        Ok(balance)
    }

    pub async fn discover_accounts<F>(
        &self,
        proton_api_client: Arc<ProtonWalletApiClient>,
        factory: F,
        discovery_account_stop_gap: Option<u32>,
        discovery_address_stop_gap: Option<usize>,
    ) -> Result<Vec<(ScriptType, u32, DerivationPath)>, Error>
    where
        F: WalletPersisterFactory,
    {
        let client = Arc::new(BlockchainClient::new(proton_api_client));
        let mut index: u32;
        let mut last_active_index: u32;
        let mut discovered_accounts: Vec<(ScriptType, u32, DerivationPath)> = Vec::new();

        let discovery_address_stop_gap = discovery_address_stop_gap.unwrap_or(ADDRESS_DISCOVERY_STOP_GAP);
        let discovery_account_stop_gap = discovery_account_stop_gap.unwrap_or(ACCOUNT_DISCOVERY_STOP_GAP);

        for script_type in ScriptType::values() {
            // Resets indexes for each script type
            index = 0;
            last_active_index = 0;

            loop {
                let derivation_path = DerivationPath::from_parts(script_type, self.network, index);

                let secp = Secp256k1::new();

                let account = if let WalletMasterKey::Xpriv(mprv) = self.key {
                    let store_key = format!("{}_{}", mprv.fingerprint(&secp), derivation_path);
                    let persister = factory.clone().build(store_key);
                    Account::new(
                        mprv,
                        self.network,
                        script_type,
                        derivation_path.clone(),
                        WalletStorage(persister),
                    )
                    .expect("Account should be valid here")
                } else if let WalletMasterKey::Xpub(xpub) = self.key {
                    let store_key = format!("{}_{}", xpub.fingerprint(), derivation_path);
                    let persister = factory.clone().build(store_key);
                    Account::new_with_xpub(
                        xpub,
                        script_type,
                        self.network,
                        derivation_path.clone(),
                        WalletStorage(persister),
                    )
                    .expect("Account should be valid here")
                } else {
                    panic!("No mprv or xpub found. This should not happen.");
                };

                let exists = AccountSyncer::new(client.clone(), Arc::new(account))
                    .check_account_existence(discovery_address_stop_gap)
                    .await?;

                // If an account has at least one output, it means that it has already been used
                if exists {
                    discovered_accounts.push((script_type, index, derivation_path));
                    last_active_index = index;
                }

                if (index - last_active_index) >= discovery_account_stop_gap {
                    break;
                }

                index += 1
            }
        }

        Ok(discovered_accounts)
    }

    pub async fn get_transactions(
        &self,
        pagination: Option<Pagination>,
        sort: Option<SortOrder>,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionDetails>, Error> {
        let pagination = pagination.unwrap_or_default();
        let async_iter = self.accounts.values().map(|account| async move {
            let wallet_lock = account.lock_wallet().await;
            let transactions = wallet_lock.transactions().collect::<Vec<_>>();

            let transactions = transactions
                .into_iter()
                .map(|tx| tx.to_transaction_details((&wallet_lock, (account.get_derivation_path()))))
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<Vec<TransactionDetails>, Error>(transactions)
        });

        let mut txs = try_join_all(async_iter)
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // Apply direction filter
        txs = match filter {
            TransactionFilter::All => txs,
            TransactionFilter::Receive => txs.into_iter().filter(|tx| !tx.is_send()).collect(),
            TransactionFilter::Send => txs.into_iter().filter(|tx| tx.is_send()).collect(),
        };

        Ok(sort_and_paginate_txs(txs, pagination, sort))
    }

    pub async fn get_transaction(
        &self,
        derivation_path: &DerivationPath,
        txid: String,
    ) -> Result<TransactionDetails, Error> {
        self.accounts
            .get(derivation_path)
            .ok_or(Error::AccountNotFound)?
            .get_transaction(txid)
            .await
    }

    pub fn get_network(&self) -> Network {
        self.network
    }

    pub fn get_fingerprint(&self) -> String {
        let secp = Secp256k1::new();
        match &self.key {
            WalletMasterKey::Xpriv(mprv) => mprv.fingerprint(&secp).to_string(),
            WalletMasterKey::Xpub(xpub) => xpub.fingerprint().to_string(),
        }
    }

    pub async fn clear_store(&self) -> Result<(), Error> {
        for a in self.get_accounts().into_iter() {
            a.clear_store().await?;
        }
        Ok(())
    }

    pub fn get_wallet_type(&self) -> WalletType {
        match self.key {
            WalletMasterKey::Xpub(_) => WalletType::WatchOnly,
            WalletMasterKey::Xpriv(_) => WalletType::OnChain,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use andromeda_common::ScriptType;
    use bitcoin::bip32::DerivationPath;

    use crate::{storage::WalletMemoryPersisterFactory, wallet::WalletType};

    const TEST_MNEMONIC: &str = "category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade";
    const XPUB: &str = "xpub6DBNnY1ewRmP4HCmfAfxod8k8V4dNXN6FcnkorJa26Pu6BpEiJh15aKGFpSLayyBXZUzw9ERcFf5z7kib4ekGW5DvGuLoE5ZExxxQqppYCf";
    const DERIVATION_PATH: &str = "m/84'/0'/0'";

    #[tokio::test]
    async fn test_create_wallet_with_xpriv() {
        let mut wallet = super::Wallet::new(super::Network::Bitcoin, TEST_MNEMONIC.to_string(), None).unwrap();

        assert_eq!(wallet.get_network(), super::Network::Bitcoin);
        assert_eq!(wallet.get_fingerprint(), "a7fd5114");
        assert!(wallet.xpub().0.is_none());
        assert!(wallet.mprv().0.is_some());
        assert_eq!(
            wallet.mprv().0.unwrap().to_string(),
            "xprv9s21ZrQH143K3mp3258jgzbiGdAnGR3t3acGhzwPf9fTGq5oF5NR64f2QxJfeZDetwrvYCVQ7FwT9ZUQujso451moCWGPfRtv1m1a39V55L"
        );
        assert_eq!(wallet.get_wallet_type(), WalletType::OnChain);

        let account = wallet.add_account(
            ScriptType::NativeSegwit,
            DerivationPath::from_str(DERIVATION_PATH).unwrap(),
            WalletMemoryPersisterFactory,
        );
        assert!(account.is_ok());

        let account = account.unwrap();
        assert_eq!(&account.get_xpub().await.unwrap().to_string(), XPUB);

        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1q8lakdutt2atnthue4j8fngj7kdqdfh6jav29ke"
        );
        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1qm3qzhvwfj3lycdxjyrd9ll8u5v0m8yd29qjgy7"
        );
        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1q28cw58duzas8k522xpjqqj5xcmpfazr0ft5juk"
        );
    }

    #[tokio::test]
    async fn test_create_wallet_with_xpub() {
        let mut wallet = super::Wallet::new_watch_only(super::Network::Bitcoin, XPUB.to_owned()).unwrap();

        assert_eq!(wallet.get_network(), super::Network::Bitcoin);
        assert_eq!(wallet.get_fingerprint(), "e0cb8774");
        assert!(wallet.xpub().0.is_some());
        assert!(wallet.mprv().0.is_none());
        assert_eq!(wallet.xpub().0.unwrap().to_string(), XPUB);
        assert_eq!(wallet.get_wallet_type(), WalletType::WatchOnly);

        let account = wallet.add_account(
            ScriptType::NativeSegwit,
            DerivationPath::from_str(DERIVATION_PATH).unwrap(),
            WalletMemoryPersisterFactory,
        );
        assert!(account.is_ok());

        let account = account.unwrap();
        assert_eq!(&account.get_xpub().await.unwrap().to_string(), XPUB);

        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1q8lakdutt2atnthue4j8fngj7kdqdfh6jav29ke"
        );
        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1qm3qzhvwfj3lycdxjyrd9ll8u5v0m8yd29qjgy7"
        );
        assert_eq!(
            account.get_next_receive_address().await.unwrap().address.to_string(),
            "bc1q28cw58duzas8k522xpjqqj5xcmpfazr0ft5juk"
        );
    }
}
