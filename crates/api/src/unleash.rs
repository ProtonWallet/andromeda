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
    use crate::{
        core::ApiClient, read_mock_file, tests::utils::common_api_client, tests::utils::setup_test_connection,
        unleash::UnleashClient,
    };
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

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

    #[tokio::test]
    async fn test_fetch_toggles_success() {
        let mock_server = MockServer::start().await;
        let req_path: String = "feature/v2/frontend".to_string();
        let contents = read_mock_file!("fetch_toggles_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = UnleashClient::new(api_client);
        let result = client.fetch_toggles().await;
        match result {
            Ok(response) => {
                assert_eq!(response.status_code, 200);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
