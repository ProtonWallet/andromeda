use std::{fmt::Debug, io};

use bitcoin::{BlockHash, Txid};

/// Errors that can happen during a sync with `Esplora`
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error related to andromeda occured: \n\t{0}")]
    ApiError(#[from] andromeda_api::error::Error),
    #[error("An error related to muon occured: \n\t{0}")]
    MuonError(#[from] andromeda_api::MuonError),
    #[error("An error related to BIP32 occured: \n\t{0}")]
    HttpResponse(u16),
    #[error("IO error during ureq response read: \n\t{0}")]
    Io(io::Error),
    #[error("No header found in ureq response")]
    NoHeader,
    #[error("Invalid number returned: \n\t{0}")]
    Parsing(#[from] std::num::ParseIntError),
    #[error("Invalid Bitcoin data returned: \n\t{0}")]
    BitcoinEncoding(#[from] bitcoin::consensus::encode::Error),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToArrayError(#[from] bitcoin::hashes::hex::HexToArrayError),
    #[error("Invalid Hex data returned: \n\t{0}")]
    HexToBytesError(#[from] bitcoin::hashes::hex::HexToBytesError),
    #[error("Transaction not found: \n\t{0}")]
    TransactionNotFound(Txid),
    #[error("Header height not found: \n\t{0}")]
    HeaderHeightNotFound(u32),
    #[error("Header hash not found: \n\t{0}")]
    HeaderHashNotFound(BlockHash),
}
