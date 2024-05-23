use bitcoin::{consensus::encode::Error as BitcoinEncodingError, hashes::hex::Error as HexError};
use muon::{Error as MuonError, RequestError as MuonRequestError, SessionError as MuonSessionError};
use serde::Deserialize;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A error from Muon occured: \n\t{0}")]
    MuonError(#[from] MuonError),
    // Cannot provide more details because session mod is private for now
    #[error("An session error occured in Muon: \n\t{0}")]
    MuonSessionError(#[from] MuonSessionError),
    #[error("A request error occured in Muon: \n\t{0}")]
    MuonRequestError(#[from] MuonRequestError),
    #[error("Bitcoin deserialize error: \n\t{0}")]
    BitcoinDeserializeError(#[from] BitcoinEncodingError),
    #[error("Error decoding hex for bitcoin: \n\t{0}")]
    HexDecoding(#[from] HexError),
    #[error("HTTP error")]
    HttpError,
    #[error("HTTP Response error")]
    ErrorCode(ResponseError),
    #[error("Response parser error")]
    DeserializeErr(String),
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ResponseError {
    pub Code: u16,
    pub Details: serde_json::Value,
    pub Error: String,
}
