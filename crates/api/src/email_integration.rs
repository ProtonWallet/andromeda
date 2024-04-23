use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, ResponseError},
    BASE_WALLET_API_V1,
};

#[derive(Clone)]
pub struct EmailIntegrationClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletBitcoinAddressLookup {
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
struct LookupBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddressLookup,
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

    pub async fn lookup_bitcoin_address(&self, email: String) -> Result<ApiWalletBitcoinAddressLookup, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/emails/lookup?Email={}", BASE_WALLET_API_V1, email),
        );

        let response = self.session.read().await.bind(request)?.send().await?;

        let parsed = response.to_json::<LookupBitcoinAddressResponseBody>()?;

        Ok(parsed.WalletBitcoinAddress)
    }

    pub async fn create_bitcoin_addresses_request(&self, email: String) -> Result<(), Error> {
        let payload = CreateBitcoinAddressRequestBody { Email: email };
        let request =
            ProtonRequest::new(Method::POST, format!("{}/emails/requests", BASE_WALLET_API_V1)).json_body(payload)?;

        let response = self.session.read().await.bind(request)?.send().await?;

        let _ = response.to_json::<CreateBitcoinAddressRequestResponseBody>()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::EmailIntegrationClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_lookup_bitcoin_address() {
        let session = common_session().await;
        let client = EmailIntegrationClient::new(session);

        let bitcoin_address = client.lookup_bitcoin_address(String::from("pro@proton.black")).await;

        println!("request done: {}", bitcoin_address.err().unwrap());
    }
}
