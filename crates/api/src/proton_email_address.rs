use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Request, Session};
use serde::Deserialize;

use crate::{error::Error, proton_response_ext::ProtonResponseExt, BASE_CORE_API_V4};

#[derive(Clone)]
pub struct ProtonEmailAddressClient {
    session: Arc<RwLock<Session>>,
}

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
    pub DomainID: String,
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

impl ProtonEmailAddressClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_proton_email_addresses(&self) -> Result<Vec<ApiProtonAddress>, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/addresses", BASE_CORE_API_V4));
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetApiProtonAddressesResponseBody>()?;

        Ok(parsed.Addresses)
    }

    pub async fn get_all_public_keys(
        &self,
        email: String,
        internal_only: Option<u8>,
    ) -> Result<Vec<ApiAllKeyAddressKey>, Error> {
        let mut request =
            ProtonRequest::new(Method::GET, format!("{}/keys/all", BASE_CORE_API_V4)).param("Email", Some(email));
        if internal_only.is_some() {
            request = request.param("InternalOnly", Some(internal_only.unwrap_or(1).to_string()));
        }

        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetApiAllKeyResponseBody>()?;

        Ok(parsed.Address.Keys)
    }
}

#[cfg(test)]
mod tests {

    use super::ProtonEmailAddressClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_proton_email_addresses() {
        let session = common_session().await;
        let client = ProtonEmailAddressClient::new(session);

        let proton_email_addresses = client.get_proton_email_addresses().await;

        println!("request done: {:?}", proton_email_addresses);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_all_public_keys() {
        let session = common_session().await;
        let client = ProtonEmailAddressClient::new(session);

        let all_public_keys = client
            .get_all_public_keys(String::from("pro@proton.black"), Some(1))
            .await;

        println!("request done: {:?}", all_public_keys);
    }
}
