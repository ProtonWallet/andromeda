use std::fmt::Debug;

use andromeda_esplora::error::Error as EsploraClientError;
use bdk_wallet::{
    chain::local_chain::CannotConnectError,
    descriptor::DescriptorError,
    keys::bip39::Error as Bip39Error,
    wallet::{
        error::{BuildFeeBumpError, CreateTxError, MiniscriptPsbtError},
        signer::SignerError,
        tx_builder::AddUtxoError,
        InsertTxError, NewOrLoadError,
    },
};
use bitcoin::{
    address::{Error as BitcoinAddressError, ParseError as BitcoinAddressParseError},
    bip32::Error as Bip32Error,
    psbt::{Error as PsbtError, ExtractTxError},
    OutPoint,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Account wasn't found")]
    AccountNotFound,
    #[error("An error occured when trying to load or create wallet: \n\t{0}")]
    NewOrLoadWallet(#[from] NewOrLoadError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    MiniscriptPsbt(#[from] MiniscriptPsbtError),
    #[error("An error related to BIP32 occured: \n\t{0:?}")]
    CreateTx(#[from] CreateTxError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    BuildFeeBump(#[from] BuildFeeBumpError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    AddUtxo(#[from] AddUtxoError),
    #[error("An error occured when signing the transaction: \n\t{0}")]
    Signer(#[from] SignerError),
    #[error("An error occured when inserting the transaction: \n\t{0}")]
    InsertTx(#[from] InsertTxError),
    #[error("Cannot connect update: update does not have a common checkpoint with the original chain.: \n\t{0}")]
    CannotConnect(#[from] CannotConnectError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    Bip32(#[from] Bip32Error),
    #[error("An error related to BIP39 occured: \n\t{0}")]
    Bip39(#[from] Bip39Error),
    #[error("An error occured in esplora client: \n\t{0}")]
    EsploraClient(#[from] EsploraClientError),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToArray(#[from] bitcoin::hashes::hex::HexToArrayError),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToBytes(#[from] bitcoin::hashes::hex::HexToBytesError),
    #[error("An error occured when parsing bitcoin address : \n\t{0}")]
    BitcoinAddressParse(#[from] BitcoinAddressParseError),
    #[error("An error related to bitcoin address occured: \n\t{0}")]
    BitcoinAddress(#[from] BitcoinAddressError),
    #[error("An error related to descriptors occured: \n\t{0}")]
    Descriptor(#[from] DescriptorError),
    #[error("An error occured when extracting tx from psbt: \n\t{0}")]
    ExtractTx(#[from] ExtractTxError),
    #[error("An error occured when interacting with PSBT: \n\t{0}")]
    Psbt(#[from] PsbtError),
    #[error("Address is invalid: {0}")]
    InvalidAddress(String),
    #[error("Data is invalid: {0:?}")]
    InvalidData(Vec<u8>),
    #[error("Transaction was not found")]
    TransactionNotFound,
    #[error("UTXO was not found: {0:?}")]
    UtxoNotFound(OutPoint),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
