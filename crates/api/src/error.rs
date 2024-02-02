use muon::request::Error as MuonError;

#[derive(Debug)]
pub enum Error {
    MuonError(MuonError),
    DeserializeError,
}

impl Into<Error> for MuonError {
    fn into(self) -> Error {
        Error::MuonError(self)
    }
}
