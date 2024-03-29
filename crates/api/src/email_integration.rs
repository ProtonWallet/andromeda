use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{error::Error, BASE_WALLET_API_V1};

#[derive(Clone)]
pub struct EmailIntegrationClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletBitcoinAddress {
    pub ID: String,
    pub Email: String,
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateBitcoinAddressRequestBody {
    pub Email: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetLookupBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddress,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CreateBitcoinAddressRequestResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

impl EmailIntegrationClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_bitcoin_address(&self, email: String) -> Result<ApiWalletBitcoinAddress, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/emails/lookup?Email={}", BASE_WALLET_API_V1, email),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let utf8_str = std::str::from_utf8(response.body()).unwrap();
        println!("{}", utf8_str);
        let parsed = response
            .to_json::<GetLookupBitcoinAddressResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletBitcoinAddress)
    }

    pub async fn create_bitcoin_addresses_request(&self, email: String) -> Result<(), Error> {
        let payload = CreateBitcoinAddressRequestBody { Email: email };
        let request = ProtonRequest::new(Method::POST, format!("{}/emails/requests", BASE_WALLET_API_V1))
            .json_body(payload)
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<CreateBitcoinAddressRequestResponseBody>()
            .map_err(|_| Error::DeserializeError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::EmailIntegrationClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_bitcoin_address() {
        let session = common_session().await;
        let client = EmailIntegrationClient::new(session);

        let bitcoin_address = client.get_bitcoin_address(String::from("pro@perey.proton.black")).await;

        println!("request done: {:?}", bitcoin_address);
    }
}
