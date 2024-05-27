use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
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

impl InviteClient {
    pub fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn send_newcomer_invite(&self, invitee_email: String) -> Result<GetInviteResponseBody, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "invites")
            .to_post_request()
            .json_body(InviteRequestBody { Email: invitee_email })?;

        let response = self.api_client.send(request).await?;

        Ok(response.parse_response::<GetInviteResponseBody>()?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{invite::InviteClient, tests, ProtonWalletApiClient, BASE_WALLET_API_V1};

    #[tokio::test]
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
        let api_client = Arc::new(ProtonWalletApiClient::from_session(
            tests::utils::setup_test_connection_raw(mock_server.uri()),
            None,
        ));
        let client = InviteClient::new(api_client);
        let result = client.send_newcomer_invite("test@pm.me".to_owned()).await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }
}
