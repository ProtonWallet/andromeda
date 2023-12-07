use crate::account::{Account, AccountConfig, SupportedBIPs};
use crate::bitcoin::Network;
use crate::error::Error;
use crate::mnemonic::Mnemonic;

use bdk::wallet::Balance as BdkBalance;

use bdk::{descriptor, Wallet as BdkWallet};
use miniscript::bitcoin::bip32::{DerivationPath, ExtendedPrivKey};
use miniscript::bitcoin::secp256k1::Secp256k1;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Wallet {
    mprv: ExtendedPrivKey,
    accounts: HashMap<DerivationPath, Account>,
    config: WalletConfig,
}

#[derive(Debug)]
pub struct WalletConfig {
    pub network: Network,
}

impl Wallet {
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

    pub fn add_account(&mut self, bip: SupportedBIPs, account_index: u32) {
        let account = Account::new(
            self.mprv,
            AccountConfig {
                bip,
                account_index,
                network: self.config.network,
            },
        )
        .unwrap();

        self.accounts.insert(account.derivation_path(), account);
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
            let account_balance = account.get_balance();

            balance.untrusted_pending += account_balance.untrusted_pending;
            balance.confirmed += account_balance.confirmed;
            balance.immature += account_balance.immature;
            balance.trusted_pending += account_balance.trusted_pending;
        }

        Ok(balance)
    }

    pub fn get_wallet(&self) -> BdkWallet {
        let external_descriptor = descriptor!(wpkh((self.mprv, Vec::new().into()))).unwrap();

        BdkWallet::new_no_persist(external_descriptor, None, self.config.network.into())
            .map_err(|e| println!("error {}", e))
            .unwrap()
    }

    pub fn get_fingerprint(&self) -> String {
        let secp = Secp256k1::new();
        self.mprv.fingerprint(&secp).to_string()
    }
}
