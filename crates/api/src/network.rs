use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{
    request::{Method, ProtonRequest, Response},
    session::Session,
};
use serde::Deserialize;

use crate::{error::Error, BASE_WALLET_API_V1};

#[derive(Clone)]
pub struct NetworkClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetNetworkResponseBody {
    pub Code: u16,
    pub Network: u8,
}

impl NetworkClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_network(&self) -> Result<u8, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/network", BASE_WALLET_API_V1));
        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| Error::MuonError(e))?;

        let parsed = response
            .to_json::<GetNetworkResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Network)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::common_session;

    use super::NetworkClient;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let session = common_session().await;
        let client = NetworkClient::new(session);

        let network = client.get_network().await;
        println!("request done: {:?}", network);
    }
}
