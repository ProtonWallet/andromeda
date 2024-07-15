use muon::http::HttpRes as ProtonResponse;
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
        match self.body_json::<T>() {
            Ok(res) => Ok(res),
            Err(parsed_response_err) => {
                // Attempt to parse the response into the error type.
                if let Ok(res) = self.body_json::<ResponseError>() {
                    return Err(Error::ErrorCode(self.status(), res));
                }

                // If parsing the known error type fails, check if the body can be read as a
                // string.
                let body = self.body().to_vec();
                let error_details = match String::from_utf8(body) {
                    Ok(text) => format!(
                        "Failed to parse response: Error: {}, Body: {}",
                        parsed_response_err, text
                    ),
                    // Directly propagate the original parsing error
                    Err(_) => parsed_response_err.to_string(),
                };

                Err(Error::Deserialize(error_details))
            }
        }
    }
}
