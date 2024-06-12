use std::sync::Arc;

use andromeda_common::Network;
use serde::Deserialize;

use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
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

impl NetworkClient {
    pub fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn get_network(&self) -> Result<Network, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "network")
            .to_get_request();

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
    use super::NetworkClient;
    use crate::tests::utils::common_api_client;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let proton_api_client = common_api_client().await;

        let client = NetworkClient::new(proton_api_client);

        let network = client.get_network().await;
        println!("request done: {:?}", network);
    }
}
