use std::fmt::Debug;

use andromeda_esplora::error::Error as EsploraClientError;
use bdk_wallet::{
    bitcoin::{
        address::ParseError as BitcoinAddressParseError,
        bip32::Error as Bip32Error,
        psbt::{Error as PsbtError, ExtractTxError},
        OutPoint,
    },
    chain::local_chain::CannotConnectError,
    descriptor::DescriptorError,
    error::{BuildFeeBumpError, MiniscriptPsbtError},
    signer::SignerError,
    tx_builder::AddUtxoError,
};
pub use bdk_wallet::{
    coin_selection::InsufficientFunds as InsufficientFundsError, error::CreateTxError, keys::bip39::Error as Bip39Error,
};
use bitcoin::address::FromScriptError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Account wasn't found")]
    AccountNotFound,
    #[error("An error occurred when trying to create persisted wallet")]
    CreateWithPersistError, /* (#[from] CreateWithPersistError) */
    #[error("An error occurred when trying to load persisted wallet")]
    LoadWithPersistError,
    #[error("Could not persist changes")]
    PersistError,
    #[error("An error related to Miniscript occurred: \n\t{0}")]
    MiniscriptPsbt(#[from] MiniscriptPsbtError),
    #[error("An error occurred when creating tx: \n\t{0:?}")]
    CreateTx(#[from] CreateTxError),
    #[error("An error occurred when bumping fees: \n\t{0}")]
    BuildFeeBump(#[from] BuildFeeBumpError),
    #[error("An error occurred when adding UTXO: \n\t{0}")]
    AddUtxo(#[from] AddUtxoError),
    #[error("An error occurred when signing the transaction: \n\t{0}")]
    Signer(#[from] SignerError),
    #[error("Cannot connect update: update does not have a common checkpoint with the original chain.: \n\t{0}")]
    CannotConnect(#[from] CannotConnectError),
    #[error("An error related to BIP32 occurred: \n\t{0}")]
    Bip32(#[from] Bip32Error),
    #[error("An error related to BIP39 occurred: \n\t{0}")]
    Bip39(#[from] Bip39Error),
    #[error("An error occurred in esplora client: \n\t{0}")]
    EsploraClient(#[from] EsploraClientError),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToArray(#[from] bitcoin::hashes::hex::HexToArrayError),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToBytes(#[from] bitcoin::hashes::hex::HexToBytesError),
    #[error("An error occurred when parsing bitcoin address : \n\t{0}")]
    BitcoinAddressParse(#[from] BitcoinAddressParseError),
    #[error("An error occurred when creating an address from script: \n\t{0}")]
    FromScript(#[from] FromScriptError),
    #[error("An error related to descriptors occurred: \n\t{0}")]
    Descriptor(#[from] DescriptorError),
    #[error("An error occurred when extracting tx from psbt: \n\t{0}")]
    ExtractTx(#[from] ExtractTxError),
    #[error("An error occurred when interacting with PSBT: \n\t{0}")]
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
    #[error("No signer found")]
    NoSignerFound,
    #[error("Invalid network")]
    InvalidNetwork,
    #[error("Message sign crypto error occurred: \n\t{0}")]
    MessageSignatureCryto(#[from] andromeda_crypto::error::Error),
    #[error("Unknown purpose in derivation path: {0}")]
    UnknownPurpose(u32),
    #[error("Script type is invalid")]
    InvalidScriptType,
    #[error("Bitcoin signed message parser error occurred: \n\t{0}")]
    BSMError(#[from] crate::bitcoin_signed_message::Error),
    #[error("Extended public key not found")]
    ExtendedPublicKeyNotFound,

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "sqlite")]
    #[error("Rusqlite persist error occurred: \n\t{0}")]
    Rusqlite(#[from] bdk_chain::rusqlite::Error),
}
