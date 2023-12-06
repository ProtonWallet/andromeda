use std::str::FromStr;

use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    chain::{ChainPosition, ConfirmationTimeAnchor},
    descriptor,
    miniscript::DescriptorPublicKey as BdkDescriptorPublicKey,
    wallet::Balance,
    KeychainKind, SignOptions,
};

use bdk::Wallet;
use miniscript::{
    bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, Transaction, Txid},
    Descriptor,
};

use crate::{bitcoin::Network, client::Client, error::Error};

#[derive(Clone, Copy, Debug)]
pub enum SupportedBIPs {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}
#[derive(Debug)]
pub struct Account {
    account_xprv: ExtendedPrivKey,
    derivation_path: DerivationPath,
    config: AccountConfig,
    wallet: Wallet,
}

#[derive(Clone, Copy, Debug)]
pub struct AccountConfig {
    pub bip: SupportedBIPs,
    pub network: Network,
    pub account_index: u32,
}

pub struct Pagination {
    skip: u32,
    take: u32,
}

impl Pagination {
    pub fn new(skip: u32, take: u32) -> Self {
        Pagination { skip, take }
    }
}

pub struct SimpleTransaction<'a> {
    pub txid: Txid,
    pub value: i64,
    pub fees: Option<u64>,
    pub confirmation: ChainPosition<&'a ConfirmationTimeAnchor>,
}

impl Account {
    fn wallet(account_xprv: ExtendedPrivKey, network: Network) -> Wallet {
        let external_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::External as u32).unwrap()];
        let internal_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::Internal as u32).unwrap()];

        let external_descriptor = descriptor!(wpkh((account_xprv, external_derivation.into()))).unwrap();
        let internal_descriptor = descriptor!(wpkh((account_xprv, internal_derivation.into()))).unwrap();

        Wallet::new_no_persist(external_descriptor, Some(internal_descriptor), network.into())
            .map_err(|e| println!("error {}", e))
            .unwrap()
    }

    pub fn get_mutable_wallet(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig) -> Result<Self, Error> {
        let secp = Secp256k1::new();

        let derivation_path = Self::gen_account_derivation_path(config.bip, config.network, config.account_index)?;

        let account_xprv = master_secret_key.derive_priv(&secp, &derivation_path).unwrap();

        Ok(Self {
            account_xprv,
            derivation_path: derivation_path.into(),
            config,
            wallet: Self::wallet(account_xprv, config.network.into()),
        })
    }

    pub fn derivation_path(&self) -> DerivationPath {
        self.derivation_path.clone()
    }

    fn gen_account_derivation_path(
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
            .unwrap(),
        );

        //  coin_type' derivation
        derivation_path.push(
            ChildNumber::from_hardened_idx(match network {
                Network::Bitcoin => 0,
                _ => 1,
            })
            .unwrap(),
        );

        // account' derivation
        derivation_path.push(ChildNumber::from_hardened_idx(account_index).map_err(|_| Error::InvalidAccountIndex)?);

        Ok(derivation_path)
    }

    pub fn public_descriptor(&self) -> BdkDescriptorPublicKey {
        let (descriptor, _, _) = descriptor!(wpkh((self.account_xprv, Vec::new().into()))).unwrap();
        match descriptor {
            Descriptor::Wpkh(pk) => pk.into_inner(),
            _ => unreachable!(),
        }
    }

    pub async fn sync(&mut self) -> Result<(), Error> {
        let client = Client::new(None)?;
        client.scan(&mut self.wallet).await?;
        Ok(())
    }

    pub fn get_balance(&self) -> Balance {
        self.wallet.get_balance()
    }

    pub fn get_transactions(&self, pagination: Pagination) -> Vec<SimpleTransaction> {
        // TODO: maybe take only confirmed transactions here?
        let ten_first_txs = self
            .wallet
            .transactions()
            .skip(pagination.skip as usize)
            .take(pagination.take as usize)
            .map(|can_tx| {
                let (sent, received) = self.wallet.spk_index().sent_and_received(can_tx.tx_node.tx);

                SimpleTransaction {
                    txid: can_tx.tx_node.txid,
                    value: received as i64 - sent as i64,
                    confirmation: can_tx.chain_position,
                    fees: match self.wallet.calculate_fee(can_tx.tx_node.tx) {
                        Ok(fees) => Some(fees),
                        _ => None,
                    },
                }
            })
            .collect::<Vec<_>>();

        ten_first_txs
    }

    pub fn get_transaction(&self, txid: String) -> Result<Transaction, Error> {
        let txid = Txid::from_str(&txid).map_err(|_| Error::InvalidTxId)?;
        let tx = match self.wallet.get_tx(txid) {
            Some(tx) => Ok(tx.tx_node.tx.clone()),
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
