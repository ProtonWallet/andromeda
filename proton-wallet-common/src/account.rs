use std::str::FromStr;

use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    chain::{ChainPosition, ConfirmationTimeAnchor},
    descriptor,
    miniscript::DescriptorPublicKey as BdkDescriptorPublicKey,
    wallet::{AddressInfo, Balance, ChangeSet},
    KeychainKind, LocalUtxo, SignOptions,
};

use bdk::Wallet;
use bdk_chain::PersistBackend;
use miniscript::{
    bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, Transaction, Txid},
    Descriptor,
};
use urlencoding::encode;

use crate::{bitcoin::Network, error::Error};

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

pub fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / 100_000_000f64
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

impl<Storage> Account<Storage>
where
    Storage: PersistBackend<ChangeSet> + Clone,
{
    fn build_wallet(account_xprv: ExtendedPrivKey, network: Network, storage: Storage) -> Wallet<Storage> {
        let external_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::External as u32).unwrap()];
        let internal_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::Internal as u32).unwrap()];

        let external_descriptor = descriptor!(wpkh((account_xprv, external_derivation.into()))).unwrap();
        let internal_descriptor = descriptor!(wpkh((account_xprv, internal_derivation.into()))).unwrap();

        Wallet::new(external_descriptor, Some(internal_descriptor), storage, network.into()).unwrap()
    }

    pub fn get_mutable_wallet(&mut self) -> &mut Wallet<Storage> {
        &mut self.wallet
    }

    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig, storage: Storage) -> Result<Self, Error> {
        let secp = Secp256k1::new();

        let derivation_path = gen_account_derivation_path(config.bip, config.network, config.account_index)?;
        let account_xprv = master_secret_key.derive_priv(&secp, &derivation_path).unwrap();

        Ok(Self {
            account_xprv,
            derivation_path: derivation_path.into(),
            storage: storage.clone(),
            wallet: Self::build_wallet(account_xprv, config.network.into(), storage),
        })
    }

    pub fn get_derivation_path(&self) -> DerivationPath {
        self.derivation_path.clone()
    }

    pub fn public_descriptor(&self) -> BdkDescriptorPublicKey {
        let (descriptor, _, _) = descriptor!(wpkh((self.account_xprv, Vec::new().into()))).unwrap();
        match descriptor {
            Descriptor::Wpkh(pk) => pk.into_inner(),
            _ => unreachable!(),
        }
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
    ) -> String {
        let address = self.get_address(index);
        let uri = format!("bitcoin:{}", address.to_string());

        let str_amount = match amount {
            Some(amount) => Some(sats_to_btc(amount).to_string()),
            _ => None,
        };

        let params_str = vec![("amount", str_amount), ("label", label), ("message", message)]
            .into_iter()
            .filter_map(move |(key, value)| match value {
                Some(value) => Some(format!("{}={}", key, encode(&value))),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join("&");

        if params_str.len() > 0 {
            return format!("{}?{}", uri, params_str);
        }

        uri
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

#[cfg(test)]
mod tests {
    use bdk::keys::bip39::Mnemonic;
    use miniscript::bitcoin::bip32::ExtendedPrivKey;

    use crate::bitcoin::Network;

    use super::{Account, AccountConfig};

    fn bitcoin_uri_setup() -> Account<()> {
        let config = AccountConfig {
            bip: super::SupportedBIPs::Bip84,
            network: Network::Testnet.into(),
            account_index: 0,
        };

        let mnemonic = Mnemonic::parse("category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade").unwrap();
        let mpriv =
            ExtendedPrivKey::new_master(miniscript::bitcoin::Network::Testnet, &mnemonic.to_seed("".to_string()));

        Account::new(mpriv.unwrap(), config, ()).unwrap()
    }

    #[test]
    fn should_return_uri_with_only_address() {
        assert_eq!(
            bitcoin_uri_setup().get_bitcoin_uri(Some(0), None, None, None),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn"
        );
    }

    #[test]
    fn should_return_uri_with_amount() {
        assert_eq!(
            bitcoin_uri_setup().get_bitcoin_uri(Some(0), Some(166727), None, None),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00166727"
        );
    }

    #[test]
    fn should_return_uri_with_encoded_label() {
        assert_eq!(
            bitcoin_uri_setup().get_bitcoin_uri(Some(0), None, Some("Fermi Pasta".to_string()), None),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?label=Fermi%20Pasta"
        );
    }

    #[test]
    fn should_return_uri_with_encoded_message() {
        assert_eq!(
            bitcoin_uri_setup().get_bitcoin_uri(Some(0), None, None, Some("Thank for your donation".to_string())),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?message=Thank%20for%20your%20donation"
        );
    }

    #[test]
    fn should_return_uri_with_all_params() {
        assert_eq!(
            bitcoin_uri_setup().get_bitcoin_uri(
                Some(0),
                Some(166727),
                Some("Fermi Pasta".to_string()),
                Some("Thank for your donation".to_string())
            ),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00166727&label=Fermi%20Pasta&message=Thank%20for%20your%20donation"
        );
    }
}
