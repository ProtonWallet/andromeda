use std::{collections::HashMap, str::FromStr};

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
use hashbrown::HashSet;
use miniscript::{
    bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, Network as BdkNetwork, Txid},
    descriptor::DescriptorSecretKey,
    Descriptor, DescriptorPublicKey,
};

use crate::{
    bitcoin::Network,
    error::Error,
    payment_link::PaymentLink,
    transactions::{DetailledTransaction, Pagination, SimpleTransaction},
    utils::sort_and_paginate_txs,
};

#[derive(Debug)]
pub struct Account<Storage> {
    derivation_path: DerivationPath,
    storage: Storage,
    wallet: Wallet<Storage>,
}

#[derive(Clone, Copy, Debug)]
pub enum ScriptType {
    Legacy,
    NestedSegwit,
    NativeSegwit,
    Taproot,
}

#[derive(Clone, Copy, Debug)]
pub struct AccountConfig {
    pub script_type: ScriptType,
    pub network: Network,
    pub account_index: u32,
    pub multisig_threshold: Option<(u32, u32)>,
}

impl AccountConfig {
    pub fn new(
        script_type: ScriptType,
        network: Network,
        account_index: u32,
        multisig_threshold: Option<(u32, u32)>,
    ) -> Self {
        Self {
            script_type,
            network,
            account_index,
            multisig_threshold,
        }
    }
}

pub fn build_account_derivation_path(config: AccountConfig) -> Result<Vec<ChildNumber>, Error> {
    let purpose = ChildNumber::from_hardened_idx(if config.multisig_threshold.is_some() {
        87 // https://bips.dev/87/
    } else {
        match config.script_type {
            ScriptType::Legacy => 44,       // https://bips.dev/44/
            ScriptType::NestedSegwit => 49, // https://bips.dev/49/
            ScriptType::NativeSegwit => 84, // https://bips.dev/84/
            ScriptType::Taproot => 86,      // https://bips.dev/86/
        }
    })
    .unwrap_or_else(|_| unreachable!("an error occured while generating child number from bip"));

    let coin_type = ChildNumber::from_hardened_idx(match config.network {
        Network::Bitcoin => 0,
        _ => 1,
    })
    .unwrap();

    let account = ChildNumber::from_hardened_idx(config.account_index).map_err(|_| Error::InvalidAccountIndex)?;

    Ok(vec![purpose, coin_type, account])
}

fn build_account_descriptors(
    account_xprv: ExtendedPrivKey,
    config: AccountConfig,
) -> Result<
    (
        (
            Descriptor<DescriptorPublicKey>,
            HashMap<DescriptorPublicKey, DescriptorSecretKey>,
            HashSet<BdkNetwork>,
        ),
        (
            Descriptor<DescriptorPublicKey>,
            HashMap<DescriptorPublicKey, DescriptorSecretKey>,
            HashSet<BdkNetwork>,
        ),
    ),
    Error,
> {
    let builder = if config.multisig_threshold.is_some() {
        todo!()
    } else {
        match config.script_type {
            ScriptType::Legacy => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(pkh(xkey)),
            ScriptType::NestedSegwit => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(sh(wpkh(xkey))),
            ScriptType::NativeSegwit => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(wpkh(xkey)),
            ScriptType::Taproot => |xkey: (ExtendedPrivKey, DerivationPath)| descriptor!(tr(xkey)),
        }
    };

    let internal = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::External as u32,
        }]
        .into(),
    ))
    .map_err(|e| e.into())?;

    let external = builder((
        account_xprv,
        vec![ChildNumber::Normal {
            index: KeychainKind::External as u32,
        }]
        .into(),
    ))
    .map_err(|e| e.into())?;

    Ok((external, internal))
}

impl<Storage> Account<Storage>
where
    Storage: PersistBackend<ChangeSet> + Clone,
{
    fn build_wallet(
        account_xprv: ExtendedPrivKey,
        config: AccountConfig,
        storage: Storage,
    ) -> Result<Wallet<Storage>, Error> {
        let (external_descriptor, internal_descriptor) = build_account_descriptors(account_xprv, config)?;

        Wallet::new(
            external_descriptor,
            Some(internal_descriptor),
            storage,
            config.network.into(),
        )
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

        let derivation_path = build_account_derivation_path(config)?;
        let account_xprv = master_secret_key
            .derive_priv(&secp, &derivation_path)
            .map_err(|e| e.into())?;

        Ok(Self {
            derivation_path: derivation_path.into(),
            storage: storage.clone(),
            wallet: Self::build_wallet(account_xprv, config, storage)?,
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
        let sign_options = sign_options.unwrap_or_default();

        self.wallet
            .sign(psbt, sign_options)
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }
}
