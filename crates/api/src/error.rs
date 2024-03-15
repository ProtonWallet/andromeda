use muon::{Error as MuonError, RequestError as MuonRequestError};

#[derive(Debug)]
pub enum Error {
    MuonError(MuonError),
    // Cannot provide more details because session mod is private for now
    MuonSessionError,
    MuonRequestError(MuonRequestError),
    DeserializeError,
    SerializeError,
    HttpError,
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
