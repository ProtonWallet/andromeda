use std::fmt::Debug;

use bdk::{descriptor::DescriptorError, keys::bip39::Error as Bip39Error, Error as BdkError};
use bitcoin::{
    address::Error as BitcoinAddressError, bip32::Error as Bip32Error, hashes::hex::Error as BitcoinHexError,
    Error as BitcoinError,
};
use esplora_client::Error as EsploraClientError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Account wasn't found")]
    AccountNotFound,
    #[error("An error from BDK occured: \n\t{0}")]
    BdkError(#[from] BdkError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    Bip32Error(#[from] Bip32Error),
    #[error("An error related to BIP39 occured: \n\t{0}")]
    Bip39Error(#[from] Bip39Error),
    #[error("An error occured in esplora client: \n\t{0}")]
    EsploraClientError(#[from] EsploraClientError),
    #[error("An error related to rust-bitcoin occured: \n\t{0}")]
    RustBitcoinError(#[from] BitcoinError),
    #[error("An error related to bitcoin hashing occured: \n\t{0}")]
    BitcoinHexError(#[from] BitcoinHexError),
    #[error("An error related to bitcoin address occured: \n\t{0}")]
    BitcoinAddressError(#[from] BitcoinAddressError),
    #[error("An error related to descriptors occured: \n\t{0}")]
    DescriptorError(#[from] DescriptorError),
    #[error("Address is invalid: {0}")]
    InvalidAddress(String),
    #[error("Data is invalid: {0:?}")]
    InvalidData(Vec<u8>),
    #[error("Transaction was not found")]
    TransactionNotFound,
}
