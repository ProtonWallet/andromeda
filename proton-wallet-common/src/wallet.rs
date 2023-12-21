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
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Wallet<Storage> {
    mprv: ExtendedPrivKey,
    accounts: HashMap<DerivationPath, Arc<Mutex<Account<Storage>>>>,
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
        let mnemonic = Mnemonic::from_string(bip39_mnemonic).unwrap();
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

    pub fn add_account(&mut self, bip: SupportedBIPs, account_index: u32, storage: Storage) -> DerivationPath {
        let account = Account::new(
            self.mprv,
            AccountConfig::new(bip, self.config.network.into(), account_index),
            storage,
        )
        .unwrap();

        let derivation_path = account.get_derivation_path();
        self.accounts
            .insert(derivation_path.clone(), Arc::new(Mutex::new(account)));

        derivation_path
    }

    pub fn get_account(&mut self, derivation_path: &DerivationPath) -> Option<&Arc<Mutex<Account<Storage>>>> {
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

        let balance = iter.fold(init, |acc, account| {
            let account_balance = account.lock().unwrap().get_balance();

            BdkBalance {
                untrusted_pending: acc.untrusted_pending + account_balance.untrusted_pending,
                confirmed: acc.confirmed + account_balance.confirmed,
                immature: acc.immature + account_balance.immature,
                trusted_pending: acc.trusted_pending + account_balance.trusted_pending,
            }
        });

        Ok(balance)
    }

    pub fn get_transactions(&self, pagination: Option<Pagination>, sorted: bool) -> Vec<SimpleTransaction> {
        let pagination = pagination.unwrap_or_default();

        let simple_txs = self
            .accounts
            .keys()
            .flat_map(|account_key| {
                let account = self.accounts.get(&account_key).unwrap();
                let account_guard = account.lock().unwrap();
                let wallet = account_guard.get_wallet();

                wallet
                    .transactions()
                    .map(|can_tx| SimpleTransaction::from_can_tx(&can_tx, &wallet, Some(account_key.clone())))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        sort_and_paginate_txs(simple_txs, pagination, sorted)
    }

    pub fn get_transaction(
        &self,
        derivation_path: &DerivationPath,
        txid: String,
    ) -> Result<DetailledTransaction, Error> {
        let account = self.accounts.get(derivation_path);

        match account {
            Some(account) => account.lock().unwrap().get_transaction(txid),
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
