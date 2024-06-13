use ::muon::Error as MuonError;
use bitcoin::{consensus::encode::Error as BitcoinEncodingError, hashes::hex::Error as HexError};
use muon::{http::StatusErr, ParseAppVersionErr};
use serde::Deserialize;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error occured in the Muon Api Version parser: \n\t{0}")]
    MuonApiVersion(#[from] ParseAppVersionErr),
    #[error("A error from Muon status: \n\t{0}")]
    MuonStatueError(#[from] StatusErr),
    #[error("A error from Muon occured: \n\t{0}")]
    MuonError(#[from] MuonError),
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
