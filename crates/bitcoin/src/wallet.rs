use core::fmt::Debug;
use std::collections::HashMap;

use andromeda_common::{Network, ScriptType};
use bdk_wallet::{
    bitcoin::{
        bip32::{DerivationPath, Xpriv},
        secp256k1::Secp256k1,
        Amount, NetworkKind,
    },
    wallet::Balance,
};
use futures::future::try_join_all;

use super::{account::Account, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    error::Error,
    mnemonic::Mnemonic,
    storage::{WalletStore, WalletStoreFactory},
    transactions::{ToTransactionDetails, TransactionDetails},
    utils::SortOrder,
};

#[derive(Debug)]
pub struct Wallet<P: WalletStore> {
    mprv: Xpriv,
    accounts: HashMap<DerivationPath, Account<P>>,
    network: Network,
}

impl<P: WalletStore> Wallet<P> {
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
    ) -> Result<Account<P>, Error>
    where
        F: WalletStoreFactory<P>,
    {
        let account = Account::new(self.mprv, self.network, script_type, derivation_path, factory)?;

        let derivation_path = account.get_derivation_path();

        self.accounts.insert(derivation_path.clone(), account.clone());

        Ok(account)
    }

    pub fn get_account(&mut self, derivation_path: &DerivationPath) -> Option<&Account<P>> {
        self.accounts.get(derivation_path)
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
            .fold(Ok(init), |acc: Result<Balance, Error>, account_balance| {
                acc.map(|acc| acc + account_balance)
            })?;

        Ok(balance)
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
}
