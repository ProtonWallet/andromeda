use crate::account::{Account, AccountConfig, SupportedBIPs};
use crate::bitcoin::Network;
use crate::error::Error;
use crate::mnemonic::Mnemonic;
use crate::transactions::{DetailledTransaction, Pagination, SimpleTransaction};
use crate::utils::sort_and_paginate_txs;

use bdk::wallet::{Balance as BdkBalance, ChangeSet};

use bdk_chain::PersistBackend;
use miniscript::bitcoin::bip32::{DerivationPath, ExtendedPrivKey};
use miniscript::bitcoin::secp256k1::Secp256k1;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Wallet<Storage> {
    mprv: ExtendedPrivKey,
    accounts: HashMap<DerivationPath, Arc<RwLock<Account<Storage>>>>,
    config: WalletConfig,
}

#[derive(Debug)]
pub struct WalletConfig {
    pub network: Network,
}

impl<Storage> Wallet<Storage>
where
    Storage: PersistBackend<ChangeSet> + Clone,
{
    pub fn new(bip39_mnemonic: String, bip38_passphrase: Option<String>, config: WalletConfig) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_string(bip39_mnemonic).map_err(|_| Error::InvalidMnemonic)?;
        let mprv = ExtendedPrivKey::new_master(
            config.network.into(),
            &mnemonic.inner().to_seed(match bip38_passphrase {
                Some(bip38_passphrase) => bip38_passphrase,
                None => "".to_string(),
            }),
        )
        .unwrap();

        Ok(Wallet {
            mprv,
            accounts: HashMap::new(),
            config,
        })
    }

    pub fn add_account(
        &mut self,
        bip: SupportedBIPs,
        account_index: u32,
        storage: Storage,
    ) -> Result<DerivationPath, Error> {
        let account = Account::new(
            self.mprv,
            AccountConfig::new(bip, self.config.network.into(), account_index),
            storage,
        )?;

        let derivation_path = account.get_derivation_path();
        self.accounts
            .insert(derivation_path.clone(), Arc::new(RwLock::new(account)));

        Ok(derivation_path)
    }

    pub fn get_account(&mut self, derivation_path: &DerivationPath) -> Option<&Arc<RwLock<Account<Storage>>>> {
        self.accounts.get(derivation_path)
    }

    pub fn get_balance(&self) -> Result<BdkBalance, Error> {
        let iter = self.accounts.values();

        let init = BdkBalance {
            untrusted_pending: 0,
            confirmed: 0,
            immature: 0,
            trusted_pending: 0,
        };

        let balance = iter.fold(Ok(init), |acc, account| {
            let account_balance = account.read().map_err(|_| Error::LockError)?.get_balance();

            match acc {
                Ok(acc) => Ok(BdkBalance {
                    untrusted_pending: acc.untrusted_pending + account_balance.untrusted_pending,
                    confirmed: acc.confirmed + account_balance.confirmed,
                    immature: acc.immature + account_balance.immature,
                    trusted_pending: acc.trusted_pending + account_balance.trusted_pending,
                }),
                _ => acc,
            }
        })?;

        Ok(balance)
    }

    pub fn get_transactions(
        &self,
        pagination: Option<Pagination>,
        sorted: bool,
    ) -> Result<Vec<SimpleTransaction>, Error> {
        let pagination = pagination.unwrap_or_default();

        let simple_txs = self
            .accounts
            .keys()
            .map(|account_key| {
                let account = self.accounts.get(&account_key).ok_or(Error::AccountNotFound)?;
                let account_guard = account.read().map_err(|_| Error::LockError)?;
                let wallet = account_guard.get_wallet();

                let txs = wallet
                    .transactions()
                    .map(|can_tx| SimpleTransaction::from_can_tx(&can_tx, &wallet, Some(account_key.clone())))
                    .collect::<Vec<_>>();

                Ok(txs)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let sorted = sort_and_paginate_txs(simple_txs, pagination, sorted);

        Ok(sorted)
    }

    pub fn get_transaction(
        &self,
        derivation_path: &DerivationPath,
        txid: String,
    ) -> Result<DetailledTransaction, Error> {
        let account = self.accounts.get(derivation_path);

        match account {
            Some(account) => account.read().map_err(|_| Error::LockError)?.get_transaction(txid),
            _ => Err(Error::InvalidAccountIndex),
        }
    }

    pub fn get_network(&self) -> Network {
        self.config.network
    }

    pub fn get_fingerprint(&self) -> String {
        let secp = Secp256k1::new();
        self.mprv.fingerprint(&secp).to_string()
    }
}
