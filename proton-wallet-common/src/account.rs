use std::str::FromStr;

use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    descriptor,
    wallet::{AddressInfo, Balance, ChangeSet},
    KeychainKind, LocalUtxo, SignOptions,
};

use bdk::Wallet;
use bdk_chain::PersistBackend;
use miniscript::bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, Txid};

use crate::{
    bitcoin::Network,
    error::Error,
    payment_link::PaymentLink,
    transactions::{DetailledTransaction, Pagination, SimpleTransaction},
    utils::sort_and_paginate_txs,
};

#[derive(Clone, Copy, Debug)]
pub enum SupportedBIPs {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

#[derive(Debug)]
pub struct Account<Storage> {
    account_xprv: ExtendedPrivKey,
    derivation_path: DerivationPath,
    storage: Storage,
    wallet: Wallet<Storage>,
}

#[derive(Clone, Copy, Debug)]
pub struct AccountConfig {
    pub bip: SupportedBIPs,
    pub network: Network,
    pub account_index: u32,
}

impl AccountConfig {
    pub fn new(bip: SupportedBIPs, network: Network, account_index: u32) -> Self {
        Self {
            bip,
            network,
            account_index,
        }
    }
}

pub fn gen_account_derivation_path(
    bip: SupportedBIPs,
    network: Network,
    account_index: u32,
) -> Result<Vec<ChildNumber>, Error> {
    let mut derivation_path = Vec::with_capacity(4);

    // purpose' derivation
    derivation_path.push(
        ChildNumber::from_hardened_idx(match bip {
            SupportedBIPs::Bip49 => 49,
            SupportedBIPs::Bip84 => 84,
            SupportedBIPs::Bip86 => 86,
            _ => 44,
        })
        .unwrap_or_else(|_| unreachable!("an error occured while generating child number from bip")),
    );

    //  coin_type' derivation
    derivation_path.push(
        ChildNumber::from_hardened_idx(match network {
            Network::Bitcoin => 0,
            _ => 1,
        })
        .unwrap_or_else(|_| unreachable!("an error occured while generating child number from network")),
    );

    // account' derivation
    derivation_path.push(ChildNumber::from_hardened_idx(account_index).map_err(|_| Error::InvalidAccountIndex)?);

    Ok(derivation_path)
}

impl<Storage> Account<Storage>
where
    Storage: PersistBackend<ChangeSet> + Clone,
{
    fn build_wallet(
        account_xprv: ExtendedPrivKey,
        network: Network,
        storage: Storage,
    ) -> Result<Wallet<Storage>, Error> {
        let external_derivation =
            vec![ChildNumber::from_normal_idx(KeychainKind::External as u32).unwrap_or_else(|_| unreachable!())];
        let internal_derivation =
            vec![ChildNumber::from_normal_idx(KeychainKind::Internal as u32).unwrap_or_else(|_| unreachable!())];

        // TODO here we shouldn't use account wpkh descriptor but rather on the account type
        let external_descriptor =
            descriptor!(wpkh((account_xprv, external_derivation.into()))).map_err(|e| e.into())?;
        let internal_descriptor =
            descriptor!(wpkh((account_xprv, internal_derivation.into()))).map_err(|e| e.into())?;

        Wallet::new(external_descriptor, Some(internal_descriptor), storage, network.into())
            .map_err(|_| Error::LoadError) // TODO: check how to implement Into<Error> for PersistBackend load error
    }

    pub fn get_mutable_wallet(&mut self) -> &mut Wallet<Storage> {
        &mut self.wallet
    }

    pub fn get_wallet(&self) -> &Wallet<Storage> {
        &self.wallet
    }

    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig, storage: Storage) -> Result<Self, Error> {
        let secp = Secp256k1::new();

        let derivation_path = gen_account_derivation_path(config.bip, config.network, config.account_index)?;
        let account_xprv = master_secret_key
            .derive_priv(&secp, &derivation_path)
            .map_err(|e| e.into())?;

        Ok(Self {
            account_xprv,
            derivation_path: derivation_path.into(),
            storage: storage.clone(),
            wallet: Self::build_wallet(account_xprv, config.network.into(), storage)?,
        })
    }

    pub fn get_derivation_path(&self) -> DerivationPath {
        self.derivation_path.clone()
    }

    pub fn get_balance(&self) -> Balance {
        self.wallet.get_balance()
    }

    pub fn get_utxos(&self) -> Vec<LocalUtxo> {
        self.wallet.list_unspent().collect()
    }

    pub fn get_storage(&self) -> Storage {
        self.storage.clone()
    }

    pub fn get_address(&mut self, index: Option<u32>) -> AddressInfo {
        match index {
            Some(index) => self.wallet.get_address(bdk::wallet::AddressIndex::Peek(index)),
            _ => self.wallet.get_address(bdk::wallet::AddressIndex::LastUnused),
        }
    }

    pub fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> PaymentLink {
        PaymentLink::new_bitcoin_uri(self, index, amount, label, message)
    }

    pub fn get_transactions(&self, pagination: Option<Pagination>, sorted: bool) -> Vec<SimpleTransaction> {
        let pagination = pagination.unwrap_or_default();

        // We first need to sort transactions by their time (last_seen for unconfirmed ones and confirmation_time for confirmed one)
        // The collection that happen here might be consuming, maybe later we need to rework this part
        let simple_txs = self
            .wallet
            .transactions()
            // account_key is not usefull in a single-account context
            .map(|can_tx| SimpleTransaction::from_can_tx(&can_tx, &self.wallet, None))
            .collect::<Vec<_>>();

        sort_and_paginate_txs(simple_txs, pagination, sorted)
    }

    pub fn get_transaction(&self, txid: String) -> Result<DetailledTransaction, Error> {
        let txid = Txid::from_str(&txid).map_err(|_| Error::InvalidTxId)?;
        let tx = match self.wallet.get_tx(txid) {
            Some(can_tx) => {
                let tx = DetailledTransaction::from_can_tx(&can_tx, &self.wallet)?;
                Ok(tx)
            }
            _ => Err(Error::TransactionNotFound),
        }?;

        Ok(tx)
    }

    pub fn sign(
        &self,
        psbt: &mut PartiallySignedTransaction,
        sign_options: Option<SignOptions>,
    ) -> Result<bool, Error> {
        let sign_options = match sign_options {
            Some(sign_options) => sign_options,
            _ => SignOptions::default(),
        };

        self.wallet
            .sign(psbt, sign_options)
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }
}
