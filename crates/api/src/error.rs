use muon::request::Error as MuonError;

#[derive(Debug)]
pub enum Error {
    MuonError(MuonError),
    DeserializeError,
    HttpError,
}

impl Into<Error> for MuonError {
    fn into(self) -> Error {
        Error::MuonError(self)
    }
}
