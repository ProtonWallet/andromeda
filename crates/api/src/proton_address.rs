use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::Deserialize;
use std::sync::Arc;

use crate::{error::Error, BASE_CORE_API_V4};

#[derive(Clone)]
pub struct ProtonAddressClient {
    session: Arc<RwLock<Session>>,
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
    pub Addresses: Option<Vec<ApiProtonAddress>>,
}

impl ProtonAddressClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_proton_addresses(&self) -> Result<Option<Vec<ApiProtonAddress>>, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/addresses", BASE_CORE_API_V4));

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
            .to_json::<GetApiProtonAddressesResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Addresses)
    }
}

#[cfg(test)]
mod tests {

    use super::ProtonAddressClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_proton_addresses() {
        let session = common_session().await;
        let client = ProtonAddressClient::new(session);

        let proton_addresses = client.get_proton_addresses().await;

        println!("request done: {:?}", proton_addresses);
    }
}
