use std::sync::Arc;

use andromeda_common::Network;
use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Session};
use serde::Deserialize;

use crate::{error::Error, proton_response_ext::ProtonResponseExt, BASE_WALLET_API_V1};

#[derive(Clone)]
pub struct NetworkClient {
    session: Arc<RwLock<Session>>,
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
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_network(&self) -> Result<Network, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/network", BASE_WALLET_API_V1));
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetNetworkResponseBody>()?;
        let network = match parsed.Network {
            0 => Network::Bitcoin,
            _ => Network::Testnet,
        };

        Ok(network)
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let session = common_session().await;
        let client = NetworkClient::new(session);

        let network = client.get_network().await;
        println!("request done: {:?}", network);
    }
}
