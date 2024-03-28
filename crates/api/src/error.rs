use muon::{Error as MuonError, RequestError as MuonRequestError};
use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    MuonError(MuonError),
    // Cannot provide more details because session mod is private for now
    MuonSessionError,
    MuonRequestError(MuonRequestError),
    DeserializeError,
    SerializeError,
    HttpError,
    ErrorCode(ResponseError),
}

#[derive(Debug, Deserialize)]
pub struct ResponseError {
    #[serde(rename = "Code")]
    pub code: u16,
    #[serde(rename = "Error")]
    pub message: String,
    #[serde(rename = "Details")]
    pub details: serde_json::Value,
}

impl Into<Error> for MuonError {
    fn into(self) -> Error {
        Error::MuonError(self)
    }
}

impl Into<Error> for MuonRequestError {
    fn into(self) -> Error {
        Error::MuonRequestError(self)
    }
}
