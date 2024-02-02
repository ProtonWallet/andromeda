use muon::{
    request::{Error as ReqError, Method, ProtonRequest, Response},
    session::Session,
};
use serde::Deserialize;

use crate::BASE_WALLET_API_V1;

pub struct NetworkClient {
    session: Session,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetNetworkResponseBody {
    pub Code: u16,
    pub Network: u8,
}

impl NetworkClient {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    pub async fn get_network(&self) -> Result<u8, ReqError> {
        let request = ProtonRequest::new(Method::GET, format!("{}/network", BASE_WALLET_API_V1));
        let response = self.session.bind(request)?.send().await?;

        let parsed = response.to_json::<GetNetworkResponseBody>()?;
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
