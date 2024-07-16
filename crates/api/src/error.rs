use std::str::Utf8Error;

pub use ::muon::{Error as MuonError, ErrorKind as MuonErrorKind};
use bitcoin::{
    consensus::encode::Error as BitcoinEncodingError,
    hashes::hex::{HexToArrayError, HexToBytesError},
};
use muon::{
    http::{Status, StatusErr},
    middleware::AuthErr,
    ParseAppVersionErr,
};
use serde::Deserialize;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A muon {0} error was caused by a non-existent auth session")]
    AuthSession(MuonErrorKind),
    #[error("A muon {0} error was caused by a failed auth refresh")]
    AuthRefresh(MuonErrorKind),
    #[error("An error occurred in the Muon App Version parser: \n\t{0}")]
    MuonAppVersion(#[from] ParseAppVersionErr),
    #[error("An error from Muon status: \n\t{0}")]
    MuonStatus(#[from] StatusErr),
    #[error("An error from Muon occurred: \n\t{0}")]
    MuonError(#[source] MuonError),
    #[error("Bitcoin deserialize error: \n\t{0}")]
    BitcoinDeserialize(#[from] BitcoinEncodingError),
    #[error("An error occurred when decoding hex to array: \n\t{0}")]
    HexToArrayDecoding(#[from] HexToArrayError),
    #[error("An error occurred when decoding hex to bytes: \n\t{0}")]
    HexToBytesErrorDecoding(#[from] HexToBytesError),
    #[error("HTTP error")]
    Http,
    #[error("HTTP Response error")]
    ErrorCode(Status, ResponseError),
    #[error("Response parser error")]
    Deserialize(String),
    #[error("Utf8 parsing error")]
    Utf8Error(#[from] Utf8Error),
}

impl From<MuonError> for Error {
    fn from(err: MuonError) -> Self {
        use std::error::Error as _;

        let Some(src) = err.source() else {
            return Error::MuonError(err);
        };

        if let Some(AuthErr::Refresh) = src.downcast_ref() {
            return Error::AuthRefresh(err.kind());
        }

        if let Some(AuthErr::Session) = src.downcast_ref() {
            return Error::AuthSession(err.kind());
        }

        Error::MuonError(err)
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ResponseError {
    pub Code: u16,
    pub Details: serde_json::Value,
    pub Error: String,
}
