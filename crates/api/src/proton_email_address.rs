use std::sync::Arc;

use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetApiAllKeyResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Address: ApiAllKeyAddress,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiAllKeyAddress {
    pub Keys: Vec<ApiAllKeyAddressKey>,
    // skipping `SignedKeyList` from API document, will add it once we need to use it
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiAllKeyAddressKey {
    pub Flags: u32,
    pub PublicKey: String,
    pub Source: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonAddress {
    pub ID: String,
    pub DomainID: Option<String>,
    pub Email: String,
    pub Status: u32,
    pub Type: u32,
    pub Receive: u32,
    pub Send: u32,
    pub DisplayName: String,
    pub Keys: Option<Vec<ApiProtonAddressKey>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonAddressKey {
    pub ID: String,
    pub Version: u32,
    pub PublicKey: String,
    pub PrivateKey: Option<String>,
    pub Token: Option<String>,
    pub Signature: Option<String>,
    pub Primary: u32,
    pub Active: u32,
    pub Flags: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetApiProtonAddressesResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Addresses: Vec<ApiProtonAddress>,
}

#[derive(Clone)]
pub struct ProtonEmailAddressClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ProtonEmailAddressClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_CORE_API_V4
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl ProtonEmailAddressClient {
    pub async fn get_proton_email_addresses(&self) -> Result<Vec<ApiProtonAddress>, Error> {
        let request = self.get("addresses");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetApiProtonAddressesResponseBody>()?;

        Ok(parsed.Addresses)
    }

    pub async fn get_all_public_keys(
        &self,
        email: String,
        internal_only: Option<u8>,
    ) -> Result<Vec<ApiAllKeyAddressKey>, Error> {
        let mut request = self.get("keys/all").query(("Email", email));
        if let Some(intenal) = internal_only {
            request = request.query(("InternalOnly", intenal.to_string()));
        }

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetApiAllKeyResponseBody>()?;

        Ok(parsed.Address.Keys)
    }
}

#[cfg(test)]
mod tests {

    use super::ProtonEmailAddressClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

    #[tokio::test]
    #[ignore]
    async fn should_get_proton_email_addresses() {
        let api_client = common_api_client().await;
        let client = ProtonEmailAddressClient::new(api_client);

        let proton_email_addresses = client.get_proton_email_addresses().await;

        println!("request done: {:?}", proton_email_addresses);
        assert!(proton_email_addresses.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_all_public_keys() {
        let api_client = common_api_client().await;
        let client = ProtonEmailAddressClient::new(api_client);

        let all_public_keys = client
            .get_all_public_keys(String::from("pro@proton.black"), Some(1))
            .await;

        println!("request done: {:?}", all_public_keys);
        assert!(all_public_keys.is_ok());
    }
}
