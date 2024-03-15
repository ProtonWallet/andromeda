use crate::settings::UserSettings;
use crate::wallet::{ApiWallet, ApiWalletAccount, ApiWalletKey, ApiWalletSettings, ApiWalletTransaction};
use crate::{error::Error, BASE_CORE_API_V4, BASE_CORE_API_V5};
use async_std::sync::RwLock;
use muon::{
    request::{Method, ProtonRequest, Response},
    session::Session,
};
use serde::Deserialize;
use std::sync::Arc;

const MAX_EVENTS_PER_POLL: usize = 50;

#[derive(Clone)]
pub struct EventClient {
    session: Arc<RwLock<Session>>,
}

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
    pub More: u32,
    pub Wallets: Option<Vec<ApiWalletEvent>>,
    pub WalletAccounts: Option<Vec<ApiWalletAccountEvent>>,
    pub WalletKeys: Option<Vec<ApiWalletKeyEvent>>,
    pub WalletSettings: Option<Vec<ApiWalletSettingsEvent>>,
    pub WalletTransactions: Option<Vec<ApiWalletTransactionsEvent>>,
    pub WalletUserSettings: Option<UserSettings>,
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

impl EventClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn collect_events(&self, latest_event_id: String) -> Result<Vec<ApiProtonEvent>, Error> {
        let mut events = Vec::with_capacity(4);
        let event = self.get_event(&latest_event_id).await?;
        let mut has_more = event.More == 1;
        let mut next_event_id = event.EventID;

        events.push(ApiProtonEvent {
            Code: event.Code,
            EventID: next_event_id,
            More: event.More,
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
                More: event.More,
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
        let request = ProtonRequest::new(Method::GET, format!("{}/events/{}", BASE_CORE_API_V5, &latest_event_id));
        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;
        let parsed = response
            .to_json::<ApiProtonEvent>()
            .map_err(|_| Error::DeserializeError)?;
        Ok(parsed)
    }

    pub async fn get_latest_event_id(&self) -> Result<String, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/events/latest", BASE_CORE_API_V4));
        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetLatestEventIDResponseBody>()
            .map_err(|_| Error::DeserializeError)?;
        Ok(parsed.EventID)
    }
}

#[cfg(test)]
mod tests {
    use super::EventClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_latest_event_id() {
        let session = common_session().await;
        let client = EventClient::new(session);

        let latest_event_id = client.get_latest_event_id().await;
        println!("request done: {:?}", latest_event_id);
    }

    #[tokio::test]
    #[ignore]
    async fn should_collect_events() {
        let session = common_session().await;
        let client = EventClient::new(session);

        let events = client
            .collect_events(String::from(
                "1EiQQQkUiKGtEZ4rE57KQcDXumDqEwXb0YNXziO3cY7kD-M9PsNhWF3zPCQ_boOx15GkqPHG3fO65UsEC-mr6w==",
            ))
            .await;
        println!("request done: {:?}", events);
    }
}
