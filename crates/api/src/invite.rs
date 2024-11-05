use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};
use muon::common::ServiceType;

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum InviteNotificationType {
    Newcomer = 1,
    EmailIntegration = 2,
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct InviteRequestBody {
    pub Email: String,
    pub Type: InviteNotificationType,
    pub InviterAddressID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct CanSendInviteResponseBody {
    pub Code: u16,
    pub CanSend: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct SendInviteResponseBody {
    pub Code: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct RemainingMonthlyInvitations {
    pub Used: u8,
    pub Available: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetRemainingMonthlyInvitationsResponseBody {
    pub Code: u16,
    pub RemainingInvitations: RemainingMonthlyInvitations,
}

#[derive(Clone)]
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
    pub async fn send_newcomer_invite(&self, invitee_email: String, inviter_address_id: String) -> Result<(), Error> {
        let request = self
            .post("invites")
            .body_json(InviteRequestBody {
                Email: invitee_email,
                Type: InviteNotificationType::Newcomer,
                InviterAddressID: inviter_address_id,
            })?
            .service_type(ServiceType::Interactive, false);

        let response = self.api_client.send(request).await?;
        response.parse_response::<SendInviteResponseBody>()?;

        Ok(())
    }

    /// Call an endpoint to check whether or not user can send an invite to the
    /// requested email. Throw if user cannot send, resolves if user can send
    pub async fn check_invite_status(
        &self,
        invitee_email: String,
        invite_notification_type: InviteNotificationType,
        inviter_address_id: String,
    ) -> Result<u8, Error> {
        let request = self
            .get("invites")
            .query(("Email", invitee_email))
            .query(("Type", (invite_notification_type as i32).to_string()))
            .query(("InviterAddressID", inviter_address_id))
            .service_type(ServiceType::Interactive, true);

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<CanSendInviteResponseBody>()?;

        Ok(parsed.CanSend)
    }

    pub async fn send_email_integration_invite(
        &self,
        invitee_email: String,
        inviter_address_id: String,
    ) -> Result<(), Error> {
        let request = self
            .post("invites")
            .body_json(InviteRequestBody {
                Email: invitee_email,
                Type: InviteNotificationType::EmailIntegration,
                InviterAddressID: inviter_address_id,
            })?
            .service_type(ServiceType::Interactive, false);
        let response = self.api_client.send(request).await?;
        response.parse_response::<SendInviteResponseBody>()?;

        Ok(())
    }

    pub async fn get_remaining_monthly_invitation(&self) -> Result<RemainingMonthlyInvitations, Error> {
        let request = self.get("invites/remaining").service_type(ServiceType::Normal, true);
        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetRemainingMonthlyInvitationsResponseBody>()?;

        Ok(parsed.RemainingInvitations)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use wiremock::{
        matchers::{method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        core::ApiClient, invite::InviteNotificationType, tests::utils::setup_test_connection, InviteClient,
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    async fn test_send_newcomer_invite_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000
            }
        );
        let req_path: String = format!("{}/invites", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        let inviter_address_id =
            "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==";
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = InviteClient::new(Arc::new(api_client));
        let result = client
            .send_newcomer_invite("test@pm.me".to_owned(), inviter_address_id.to_owned())
            .await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
    async fn test_get_invite_status_newcomer_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "CanSend": 1
            }
        );
        let req_path: String = format!("{}/invites", BASE_WALLET_API_V1);
        let response: ResponseTemplate = ResponseTemplate::new(200).set_body_json(response_body);
        let inviter_address_id =
            "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==";
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Email", "test@pm.me"))
            .and(query_param("Type", "1"))
            .and(query_param("InviterAddressID", inviter_address_id))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = InviteClient::new(Arc::new(api_client));
        let result = client
            .check_invite_status(
                "test@pm.me".to_owned(),
                InviteNotificationType::Newcomer,
                inviter_address_id.to_owned(),
            )
            .await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
    async fn test_get_invite_status_email_integration_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "CanSend": 1
            }
        );
        let req_path: String = format!("{}/invites", BASE_WALLET_API_V1);
        let response: ResponseTemplate = ResponseTemplate::new(200).set_body_json(response_body);
        let inviter_address_id =
            "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==";
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Email", "test@pm.me"))
            .and(query_param("Type", "2"))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = InviteClient::new(Arc::new(api_client));
        let result = client
            .check_invite_status(
                "test@pm.me".to_owned(),
                InviteNotificationType::EmailIntegration,
                inviter_address_id.to_owned(),
            )
            .await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
    async fn test_send_email_integration_invitation_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
            }
        );
        let req_path: String = format!("{}/invites", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        let inviter_address_id =
            "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==";
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = InviteClient::new(Arc::new(api_client));
        let result = client
            .send_email_integration_invite("test@pm.me".to_owned(), inviter_address_id.to_owned())
            .await;
        match result {
            Ok(_) => return,
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
    async fn test_get_remaining_monthly_invitation() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "RemainingInvitations": {
                    "Used": 2,
                    "Available": 1
                }
            }
        );
        let req_path: String = format!("{}/invites/remaining", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);

        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = InviteClient::new(Arc::new(api_client));

        let result = client.get_remaining_monthly_invitation().await.unwrap();

        assert_eq!(result.clone().Available, 1);
        assert_eq!(result.clone().Used, 2);
    }
}
