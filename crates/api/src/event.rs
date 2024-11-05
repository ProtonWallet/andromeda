use std::sync::Arc;

use serde::Deserialize;

use crate::{
    contacts::ApiContactEmails,
    core::{ApiClient, ProtonResponseExt, ToProtonRequest},
    error::Error,
    proton_users::{ProtonUser, ProtonUserSettings},
    settings::UserSettings,
    wallet::{ApiWallet, ApiWalletAccount, ApiWalletKey, ApiWalletSettings, ApiWalletTransaction},
    ProtonWalletApiClient, BASE_CORE_API_V4, BASE_CORE_API_V5,
};
use muon::common::ServiceType;

const MAX_EVENTS_PER_POLL: usize = 50;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetLatestEventIDResponseBody {
    pub Code: u16,
    pub EventID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonEvent {
    pub Code: u16,
    pub EventID: String,
    pub Refresh: u32,
    pub More: u32,
    pub ContactEmails: Option<Vec<ApiContactsEmailEvent>>,
    pub Wallets: Option<Vec<ApiWalletEvent>>,
    pub WalletAccounts: Option<Vec<ApiWalletAccountEvent>>,
    pub WalletKeys: Option<Vec<ApiWalletKeyEvent>>,
    pub WalletSettings: Option<Vec<ApiWalletSettingsEvent>>,
    pub WalletTransactions: Option<Vec<ApiWalletTransactionsEvent>>,
    pub WalletUserSettings: Option<UserSettings>,
    pub User: Option<ProtonUser>,
    pub UserSettings: Option<ProtonUserSettings>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiContactsEmailEvent {
    pub ID: String,
    pub Action: u32,
    pub ContactEmail: Option<ApiContactEmails>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletEvent {
    pub ID: String,
    pub Action: u32,
    pub Wallet: Option<ApiWallet>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletAccountEvent {
    pub ID: String,
    pub Action: u32,
    pub WalletAccount: Option<ApiWalletAccount>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletKeyEvent {
    pub ID: String,
    pub Action: u32,
    pub WalletKey: Option<ApiWalletKey>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletSettingsEvent {
    pub ID: String,
    pub Action: u32,
    pub WalletSettings: Option<ApiWalletSettings>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletTransactionsEvent {
    pub ID: String,
    pub Action: u32,
    pub WalletTransaction: Option<ApiWalletTransaction>,
}

#[derive(Clone)]
pub struct EventClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for EventClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_CORE_API_V4
    }
}

impl EventClient {
    pub async fn collect_events(&self, latest_event_id: String) -> Result<Vec<ApiProtonEvent>, Error> {
        let mut events = Vec::with_capacity(4);
        let event = self.get_event(&latest_event_id).await?;
        let mut has_more = event.More == 1;
        let mut next_event_id = event.EventID;

        events.push(ApiProtonEvent {
            Code: event.Code,
            EventID: next_event_id,
            Refresh: event.Refresh,
            More: event.More,
            ContactEmails: event.ContactEmails,
            Wallets: event.Wallets,
            WalletAccounts: event.WalletAccounts,
            WalletKeys: event.WalletKeys,
            WalletSettings: event.WalletSettings,
            WalletTransactions: event.WalletTransactions,
            WalletUserSettings: event.WalletUserSettings,
            User: event.User,
            UserSettings: event.UserSettings,
        });
        let mut num_collected = 0_usize;

        while has_more {
            num_collected += 1;
            if num_collected >= MAX_EVENTS_PER_POLL {
                return Ok(events);
            }

            let event = self.get_event(&latest_event_id).await?;
            has_more = event.More == 1;
            next_event_id = event.EventID;
            events.push(ApiProtonEvent {
                Code: event.Code,
                EventID: next_event_id,
                Refresh: event.Refresh,
                More: event.More,
                ContactEmails: event.ContactEmails,
                Wallets: event.Wallets,
                WalletAccounts: event.WalletAccounts,
                WalletKeys: event.WalletKeys,
                WalletSettings: event.WalletSettings,
                WalletTransactions: event.WalletTransactions,
                WalletUserSettings: event.WalletUserSettings,
                User: event.User,
                UserSettings: event.UserSettings,
            });
        }
        Ok(events)
    }

    pub async fn get_event(&self, latest_event_id: &str) -> Result<ApiProtonEvent, Error> {
        let request = self
            .build_request(BASE_CORE_API_V5, format!("events/{}", &latest_event_id))
            .to_get_request()
            .service_type(ServiceType::Interactive, true);

        let response = self.api_client.send(request).await?;
        response.parse_response::<ApiProtonEvent>()
    }

    pub async fn get_latest_event_id(&self) -> Result<String, Error> {
        let request = self.get("events/latest").service_type(ServiceType::Interactive, true);

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetLatestEventIDResponseBody>()?;

        Ok(parsed.EventID)
    }
}

#[cfg(test)]
mod tests {

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::EventClient;
    use crate::{
        core::ApiClient,
        read_mock_file,
        tests::utils::{common_api_client, setup_test_connection_arc},
        BASE_CORE_API_V5,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_latest_event_id() {
        let api_client = common_api_client().await;
        let client = EventClient::new(api_client);

        let latest_event_id = client.get_latest_event_id().await;
        println!("request done: {:?}", latest_event_id);
        assert!(latest_event_id.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_collect_events() {
        let api_client = common_api_client().await;
        let client = EventClient::new(api_client);

        let events = client
            .collect_events(String::from(
                "1EiQQQkUiKGtEZ4rE57KQcDXumDqEwXb0YNXziO3cY7kD-M9PsNhWF3zPCQ_boOx15GkqPHG3fO65UsEC-mr6w==",
            ))
            .await;
        println!("request done: {:?}", events);
        assert!(events.is_ok());
    }

    #[tokio::test]
    async fn test_get_event_1000() {
        let contents = read_mock_file!("get_events_1000_body");
        assert!(!contents.is_empty());
        let latest_event_id = "latest_event_id";
        let req_path: String = format!("{}/events/{}", BASE_CORE_API_V5, latest_event_id);
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let event_client = EventClient::new(api_client);
        let events = event_client.get_event(latest_event_id).await.unwrap();

        assert_eq!(events.Code, 1000);
        assert_eq!(events.EventID, "ACXDmTaBub14w==");
        assert_eq!(events.Refresh, 0);
        assert_eq!(events.More, 0);
        assert!(events.ContactEmails.is_some());
        assert!(events.Wallets.is_some());
        assert!(events.WalletAccounts.is_some());
        assert!(events.WalletKeys.is_some());
        assert!(events.WalletSettings.is_some());
        assert!(events.WalletTransactions.is_some());
        assert!(events.WalletUserSettings.is_some());
    }
}
