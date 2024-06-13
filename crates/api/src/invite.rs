use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct InviteRequestBody {
    pub Email: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetInviteResponseBody {
    pub Code: u16,
}

pub struct InviteClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for InviteClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }
}

impl InviteClient {
    pub async fn send_newcomer_invite(&self, invitee_email: String) -> Result<GetInviteResponseBody, Error> {
        let request = self
            .post("invites")
            .body_json(InviteRequestBody { Email: invitee_email })?;

        let response = self.api_client.send(request).await?;

        Ok(response.parse_response::<GetInviteResponseBody>()?)
    }
}

#[cfg(test)]
mod tests {

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{core::ApiClient, tests, InviteClient, BASE_WALLET_API_V1};

    #[tokio::test]
    #[cfg(feature = "allow-dangerous-env")]
    async fn send_newcomer_invite_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000
            }
        );
        let req_path: String = format!("{}/invites", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = tests::utils::setup_test_connection(mock_server.uri());
        let client = InviteClient::new(api_client);
        let result = client.send_newcomer_invite("test@pm.me".to_owned()).await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }
}
