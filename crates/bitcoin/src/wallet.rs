use core::fmt::Debug;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use andromeda_common::{Network, ScriptType};
use bdk::{database::BatchDatabase, Balance};
use futures::future;
use miniscript::bitcoin::{
    bip32::{DerivationPath, ExtendedPrivKey},
    secp256k1::Secp256k1,
};

use super::{account::Account, transactions::Pagination, utils::sort_and_paginate_txs};
use crate::{
    error::Error,
    mnemonic::Mnemonic,
    transactions::{SimpleTransaction, TransactionDetails},
    utils::SortOrder,
};

#[derive(Debug)]
pub struct Wallet<Storage>
where
    Storage: BatchDatabase,
{
    mprv: ExtendedPrivKey,
    accounts: HashMap<DerivationPath, Arc<RwLock<Account<Storage>>>>,
    network: Network,
}

impl<Storage> Wallet<Storage>
where
    Storage: BatchDatabase + Debug,
{
    pub fn new(network: Network, bip39_mnemonic: String, bip38_passphrase: Option<String>) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_string(bip39_mnemonic).map_err(|_| Error::InvalidMnemonic)?;
        let mprv = ExtendedPrivKey::new_master(
            network.into(),
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

    pub fn new_with_accounts(
        network: Network,
        bip39_mnemonic: String,
        bip38_passphrase: Option<String>,
        accounts: Vec<(ScriptType, DerivationPath, Storage)>,
    ) -> Result<Self, Error> {
        let mut wallet = Self::new(network, bip39_mnemonic, bip38_passphrase)?;

        for (script_type, derivation_path, storage) in accounts {
            wallet.add_account(script_type, derivation_path, storage)?;
        }

        Ok(wallet)
    }

    pub fn add_account(
        &mut self,
        scrip_type: ScriptType,
        derivation_path: DerivationPath,
        storage: Storage,
    ) -> Result<DerivationPath, Error> {
        let account = Account::new(self.mprv, self.network, scrip_type, derivation_path, storage)?;

        let derivation_path = account.get_derivation_path();
        self.accounts
            .insert(derivation_path.clone(), Arc::new(RwLock::new(account)));

        Ok(derivation_path)
    }

    pub fn get_account(&mut self, derivation_path: &DerivationPath) -> Option<&Arc<RwLock<Account<Storage>>>> {
        self.accounts.get(derivation_path)
    }

    pub async fn get_balance(&self) -> Result<Balance, Error> {
        let async_iter = self.accounts.keys().map(|account_key| async move {
            let account = self.accounts.get(&account_key).ok_or(Error::AccountNotFound)?;
            let account_guard = account.read().expect("lock");
            account_guard.get_balance()
        });

        let account_balances = future::try_join_all(async_iter).await?;

        let init = Balance {
            untrusted_pending: 0,
            confirmed: 0,
            immature: 0,
            trusted_pending: 0,
        };

        let balance = account_balances
            .into_iter()
            .fold(Ok(init), |acc, account_balance| match acc {
                Ok(acc) => Ok(Balance {
                    untrusted_pending: acc.untrusted_pending + account_balance.untrusted_pending,
                    confirmed: acc.confirmed + account_balance.confirmed,
                    immature: acc.immature + account_balance.immature,
                    trusted_pending: acc.trusted_pending + account_balance.trusted_pending,
                }),
                _ => acc,
            })?;

        Ok(balance)
    }

    pub async fn get_transactions(
        &self,
        pagination: Option<Pagination>,
        sort: Option<SortOrder>,
    ) -> Result<Vec<SimpleTransaction>, Error> {
        let pagination = pagination.unwrap_or_default();

        let simple_txs = self
            .accounts
            .keys()
            .map(|account_key| {
                let account = self.accounts.get(&account_key).ok_or(Error::AccountNotFound)?;
                let account_guard = account.read().expect("lock");
                let wallet = account_guard.get_wallet();

                let transactions = wallet.list_transactions(true).map_err(|e| e.into())?;

                let transactions = transactions
                    .into_iter()
                    .map(|tx| SimpleTransaction::from_detailled_tx(tx, Some(account_key.clone())))
                    .collect::<Vec<_>>();

                Ok(transactions)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let simple_txs = simple_txs.into_iter().flatten().collect::<Vec<_>>();

        Ok(sort_and_paginate_txs(simple_txs, pagination, sort))
    }

    pub async fn get_transaction(
        &self,
        derivation_path: &DerivationPath,
        txid: String,
    ) -> Result<TransactionDetails, Error> {
        let account = self.accounts.get(derivation_path);

        match account {
            Some(account) => account.read().expect("lock").get_transaction(txid),
            _ => Err(Error::InvalidAccountIndex),
        }
    }

    pub fn get_network(&self) -> Network {
        self.network
    }

    pub fn get_fingerprint(&self) -> String {
        let secp = Secp256k1::new();
        self.mprv.fingerprint(&secp).to_string()
    }
}
