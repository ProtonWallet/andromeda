use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::Deserialize;

use crate::{error::Error, settings::FiatCurrency, BASE_WALLET_API_V1};

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

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetLookupBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddress,
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

        let bitcoin_address = client
            .get_bitcoin_address(String::from("pro@perkin.proton.black"))
            .await;

        println!("request done: {:?}", bitcoin_address);
    }
}
