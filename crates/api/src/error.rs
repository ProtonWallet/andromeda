use muon::{request::Error as MuonError, session::Error as MuonSessionError};

#[derive(Debug)]
pub enum Error {
    MuonError(MuonError),
    MuonSessionError(MuonSessionError),
    DeserializeError,
    HttpError,
}

impl Into<Error> for MuonError {
    fn into(self) -> Error {
        Error::MuonError(self)
    }
}

impl Into<Error> for MuonSessionError {
    fn into(self) -> Error {
        Error::MuonSessionError(self)
    }
}
