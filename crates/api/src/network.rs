use std::sync::Arc;

use andromeda_common::Network;
use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Clone)]
pub struct NetworkClient {
    api_client: Arc<ProtonWalletApiClient>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetNetworkResponseBody {
    //TODO:: code need to be used. remove all #[allow(dead_code)]
    #[allow(dead_code)]
    pub Code: u16,
    pub Network: u8,
}

impl ApiClient for NetworkClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl NetworkClient {
    pub async fn get_network(&self) -> Result<Network, Error> {
        let request = self.get("network");
        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetNetworkResponseBody>()?;
        let network = match parsed.Network {
            0 => Network::Bitcoin,
            1 => Network::Testnet,
            2 => Network::Signet,
            _ => Network::Regtest,
        };

        Ok(network)
    }
}

#[cfg(test)]
mod tests {
    use muon::env::EnvId;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::NetworkClient;
    use crate::{
        core::ApiClient,
        tests::utils::{common_api_client, setup_test_connection},
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let proton_api_client = common_api_client().await;

        let client = NetworkClient::new(proton_api_client);

        let network = client.get_network().await;
        println!("request done: {:?}", network);
    }

    #[test]
    fn test_parse_env_id() {
        let env: EnvId = "prod".parse().unwrap();
        assert!(matches!(env, EnvId::Prod));

        let env: EnvId = "atlas".parse().unwrap();
        assert!(matches!(env, EnvId::Atlas(None)));

        let env: EnvId = "atlas:scientist".parse().unwrap();
        assert!(matches!(env, EnvId::Atlas(Some(name)) if name == "scientist"));
    }

    #[tokio::test]
    async fn test_get_network_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Network": 0
        });

        let req_path: String = format!("{}/network", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let network_client = NetworkClient::new(api_client);
        let res = network_client.get_network().await;
        println!("test_get_network_1000 done: {:?}", res);
        assert!(res.is_ok());
        assert!(matches!(res.unwrap(), super::Network::Bitcoin));
        let unmatched_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(unmatched_requests.len(), 1, "There should be no unmatched requests");
    }

    #[tokio::test]
    async fn test_get_network_timeout() {
        let mock_server = MockServer::start().await;
        let req_path: String = format!("{}/network", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(32));
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let network_client = NetworkClient::new(api_client);
        let res = network_client.get_network().await;
        println!("test_get_network_timeout done: {:?}", res);
        assert!(res.is_err());
        let unmatched_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(unmatched_requests.len(), 1, "There should be no unmatched requests");
    }
}
