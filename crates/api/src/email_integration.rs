use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};
use muon::common::ServiceType;

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Clone)]
pub struct EmailIntegrationClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for EmailIntegrationClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }
}

impl EmailIntegrationClient {
    pub async fn lookup_bitcoin_address(&self, email: String) -> Result<ApiWalletBitcoinAddressLookup, Error> {
        let request = self
            .get("emails/lookup")
            .query(("Email", email))
            .service_type(ServiceType::Interactive, true);

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<LookupBitcoinAddressResponseBody>()?;

        Ok(parsed.WalletBitcoinAddress)
    }

    pub async fn create_bitcoin_addresses_request(&self, email: String) -> Result<(), Error> {
        let payload = CreateBitcoinAddressRequestBody { Email: email };

        let request = self
            .post("emails/requests")
            .body_json(payload)?
            .service_type(ServiceType::Interactive, false);

        let response = self.api_client.send(request).await?;
        response.parse_response::<CreateBitcoinAddressRequestResponseBody>()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::EmailIntegrationClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

    #[tokio::test]
    #[ignore]
    async fn should_lookup_bitcoin_address() {
        let api_client = common_api_client().await;
        let client = EmailIntegrationClient::new(api_client);

        let bitcoin_address = client.lookup_bitcoin_address(String::from("pro@proton.black")).await;

        println!("request done: {:?}", bitcoin_address);
        assert!(bitcoin_address.is_ok());
    }
}
