use core::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

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
use futures::future::try_join_all;

use super::{account::Account, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    account_trait::AccessWallet,
    blockchain_client::BlockchainClient,
    error::Error,
    mnemonic::Mnemonic,
    storage::{WalletPersisterFactory, WalletStorage},
    transactions::{ToTransactionDetails, TransactionDetails},
    utils::SortOrder,
};

const ACCOUNT_DISCOVERY_STOP_GAP: u32 = 2;
const ADDRESS_DISCOVERY_STOP_GAP: usize = 10;

#[derive(Debug)]
pub struct Wallet {
    mprv: Xpriv,
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
            mprv,
            accounts: HashMap::new(),
            network,
        })
    }

    pub fn mprv(&self) -> (Xpriv, Network) {
        (self.mprv, self.network)
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
        let store_key = format!("{}_{}", self.mprv.fingerprint(&secp), derivation_path);
        let persister = factory.build(store_key);

        let account = Account::new(
            self.mprv,
            self.network,
            script_type,
            derivation_path,
            WalletStorage(persister),
        )?;

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
        proton_api_client: ProtonWalletApiClient,
        factory: F,
        discovery_account_stop_gap: Option<u32>,
        discovery_address_stop_gap: Option<usize>,
    ) -> Result<Vec<(ScriptType, u32, DerivationPath)>, Error>
    where
        F: WalletPersisterFactory,
    {
        let client = BlockchainClient::new(proton_api_client);
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
                let store_key = format!("{}_{}", self.mprv.fingerprint(&secp), derivation_path);
                let persister = factory.clone().build(store_key);

                let account = Account::new(
                    self.mprv,
                    self.network,
                    script_type,
                    derivation_path.clone(),
                    WalletStorage(persister),
                )
                .expect("Account should be valid here");

                let exists = client
                    .check_account_existence(account.get_wallet().await, discovery_address_stop_gap)
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
    ) -> Result<Vec<TransactionDetails>, Error> {
        let pagination = pagination.unwrap_or_default();

        let async_iter = self.accounts.values().map(|account| async move {
            let wallet_lock = account.get_wallet().await;
            let transactions = wallet_lock.transactions().collect::<Vec<_>>();

            let transactions = transactions
                .into_iter()
                .map(|tx| tx.to_transaction_details((&wallet_lock, (account.get_derivation_path()))))
                .collect::<Result<Vec<_>, _>>()?;

            Ok::<Vec<TransactionDetails>, Error>(transactions)
        });

        let txs = try_join_all(async_iter)
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

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
        self.mprv.fingerprint(&secp).to_string()
    }

    pub async fn clear_store(&self) -> Result<(), Error> {
        for a in self.get_accounts().into_iter() {
            a.clear_store().await?;
        }
        Ok(())
    }
}
