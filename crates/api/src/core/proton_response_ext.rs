use muon::{Error as MuonError, ProtonResponse};
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
        let response_status = self.status();

        let handle_error = |response_parse_error: Option<MuonError>| -> Result<T, Error> {
            // Attempt to parse the response into the error type.
            if let Some(parsed_error_payload) = self.body_json::<ResponseError>().ok() {
                return Err(Error::ErrorCode(response_status, parsed_error_payload));
            }

            match response_parse_error {
                Some(parsing_error) => {
                    // If parsing the known error type fails, check if the body can be read as a
                    // string.
                    let body = self.body().to_vec();

                    // We either return details about the parsing error with the body as string
                    let error_details = match String::from_utf8(body) {
                        Ok(text) => format!("Failed to parse response: Error: {}, Body: {}", parsing_error, text),
                        // Or just the parsing error
                        Err(_) => parsing_error.to_string(),
                    };

                    Err(Error::Deserialize(error_details))
                }
                None => Err(Error::ErrorCode(response_status, ResponseError::default())),
            }
        };

        if response_status.is_client_error() || response_status.is_server_error() {
            return handle_error(None);
        }

        match self.body_json::<T>() {
            Ok(res) => Ok(res),
            Err(response_parse_error) => handle_error(Some(response_parse_error)),
        }
    }
}
