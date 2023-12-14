use crate::account::{Account, AccountConfig, SupportedBIPs};
use crate::bitcoin::Network;
use crate::error::Error;
use crate::mnemonic::Mnemonic;

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
        let mut iter = self.accounts.values();

        let mut balance = BdkBalance {
            untrusted_pending: 0,
            confirmed: 0,
            immature: 0,
            trusted_pending: 0,
        };

        while let Some(account) = iter.next() {
            let account_balance = account.lock().unwrap().get_balance();

            balance.untrusted_pending += account_balance.untrusted_pending;
            balance.confirmed += account_balance.confirmed;
            balance.immature += account_balance.immature;
            balance.trusted_pending += account_balance.trusted_pending;
        }

        Ok(balance)
    }

    pub fn get_network(&self) -> Network {
        self.config.network
    }

    pub fn get_fingerprint(&self) -> String {
        let secp = Secp256k1::new();
        self.mprv.fingerprint(&secp).to_string()
    }
}
