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
    use muon::EnvId;

    use super::NetworkClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

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
}
