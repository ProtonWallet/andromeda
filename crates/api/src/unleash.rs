use std::sync::Arc;

use crate::{core::ApiClient, error::Error, ProtonWalletApiClient};

#[derive(Debug)]
pub struct UnleashResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct UnleashClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for UnleashClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        ""
    }
}

impl UnleashClient {
    pub async fn fetch_toggles(&self) -> Result<UnleashResponse, Error> {
        let request = self.get("feature/v2/frontend");
        let response = self.api_client.send(request).await?;
        let status_code = response.status();
        let body = response.body();
        Ok(UnleashResponse {
            status_code: status_code.as_u16(),
            body: body.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::ApiClient, tests::utils::common_api_client, unleash::UnleashClient};

    #[tokio::test]
    #[ignore]
    async fn should_get_raw_transaction() {
        let api_client = common_api_client().await;
        let client = UnleashClient::new(api_client);
        let unleash_response = client.fetch_toggles().await;
        assert!(unleash_response.is_ok());
        let response = unleash_response.unwrap();
        assert!(response.status_code == 200);
        println!("request done code: {:?}", response.status_code);
    }
}
