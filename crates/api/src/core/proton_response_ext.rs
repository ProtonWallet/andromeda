use muon::{ProtonResponse, Response};
use serde::de::DeserializeOwned;

use crate::error::{Error, ResponseError};

pub trait ProtonResponseExt {
    fn parse_response<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned + std::fmt::Debug;
}

impl ProtonResponseExt for ProtonResponse {
    fn parse_response<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        // Attempt to parse the response into the expected type.
        let parsed_response = self.to_json::<T>();
        if let Ok(res) = parsed_response {
            return Ok(res);
        }

        if let Ok(res) = self.to_json::<ResponseError>() {
            return Err(Error::ErrorCode(res));
        }

        // If parsing the known error type fails, check if the body can be read as a
        // string.
        let body = self.body().to_vec();
        match String::from_utf8(body) {
            Ok(text) => Err(Error::DeserializeErr(format!("Failed to parse response: {}", text))),
            Err(_) => Err(Error::from(parsed_response.unwrap_err())), // Directly propagate the original parsing error
        }
    }
}
