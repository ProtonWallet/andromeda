use std::sync::Arc;

use serde::Deserialize;

use crate::{
    contacts::ApiContactEmails,
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    settings::UserSettings,
    wallet::{ApiWallet, ApiWalletAccount, ApiWalletKey, ApiWalletSettings, ApiWalletTransaction},
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

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
            });
        }
        Ok(events)
    }

    pub async fn get_event(&self, latest_event_id: &str) -> Result<ApiProtonEvent, Error> {
        let request = self.get(format!("events/{}", &latest_event_id));

        let response = self.api_client.send(request).await?;
        response.parse_response::<ApiProtonEvent>()
    }

    pub async fn get_latest_event_id(&self) -> Result<String, Error> {
        let request = self.get("events/latest");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetLatestEventIDResponseBody>()?;

        Ok(parsed.EventID)
    }
}

#[cfg(test)]
mod tests {
    use super::EventClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

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
}
