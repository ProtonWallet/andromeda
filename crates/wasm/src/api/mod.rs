use andromeda_api::{self, ApiConfig, Auth, ProtonWalletApiClient};
use bitcoin_address::WasmBitcoinAddressClient;
use email_integration::WasmEmailIntegrationClient;
use exchange_rate::WasmExchangeRateClient;
use invite::WasmInviteClient;
use network::WasmNetworkClient;
use payment_gateway::WasmPaymentGatewayClient;
use price_graph::WasmPriceGraphClient;
use settings::WasmSettingsClient;
use wallet::WasmWalletClient;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

mod bitcoin_address;
mod email_integration;
mod env;
mod exchange_rate;
mod invite;
mod network;
mod payment_gateway;
mod price_graph;
mod settings;
mod wallet;

#[wasm_bindgen(getter_with_clone)]
pub struct WasmAuthData {
    pub uid: String,
    pub access: String,
    pub refresh: String,
    pub scopes: Vec<String>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmProtonWalletApiClient(ProtonWalletApiClient);

impl From<&WasmProtonWalletApiClient> for ProtonWalletApiClient {
    fn from(value: &WasmProtonWalletApiClient) -> Self {
        value.clone().0
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiClients {
    pub exchange_rate: WasmExchangeRateClient,
    pub email_integration: WasmEmailIntegrationClient,
    pub bitcoin_address: WasmBitcoinAddressClient,
    pub payment_gateway: WasmPaymentGatewayClient,
    pub price_graph: WasmPriceGraphClient,
    pub settings: WasmSettingsClient,
    pub network: WasmNetworkClient,
    pub invite: WasmInviteClient,
    pub wallet: WasmWalletClient,
}

#[wasm_bindgen]
impl WasmProtonWalletApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        app_version: String,
        user_agent: String,
        user_id_str: Option<String>,
        uid_str: Option<String>,
        origin: Option<String>,
        url_prefix: Option<String>,
    ) -> Result<WasmProtonWalletApiClient, js_sys::Error> {
        let auth = match (user_id_str.as_deref(), uid_str.as_deref()) {
            (Some(user_id), Some(uid)) => Auth::external(user_id.to_string(), uid.to_string()),
            _ => Auth::None,
        };
        let config = ApiConfig {
            spec: (app_version, user_agent),
            auth: Some(auth),
            env: origin,
            url_prefix,
            store: None,
            proxy: None,
        };

        let client = ProtonWalletApiClient::from_config(config).map_err(|e| e.to_js_error())?;
        Ok(WasmProtonWalletApiClient(client))
    }

    #[wasm_bindgen]
    pub fn clients(&self) -> WasmApiClients {
        let clients = self.0.clients();

        WasmApiClients {
            exchange_rate: WasmExchangeRateClient::from(clients.exchange_rate),
            email_integration: WasmEmailIntegrationClient::from(clients.email_integration),
            bitcoin_address: WasmBitcoinAddressClient::from(clients.bitcoin_address),
            payment_gateway: WasmPaymentGatewayClient::from(clients.payment_gateway),
            price_graph: WasmPriceGraphClient::from(clients.price_graph),
            settings: WasmSettingsClient::from(clients.settings),
            network: WasmNetworkClient::from(clients.network),
            invite: WasmInviteClient::from(clients.invite),
            wallet: WasmWalletClient::from(clients.wallet),
        }
    }
}

#[cfg(test)]
mod tests {
    use andromeda_api::tests::utils::test_spec;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::WasmProtonWalletApiClient;

    #[wasm_bindgen_test]
    #[ignore]
    #[allow(dead_code)]
    async fn should_create_pw_api_client() {
        let client = WasmProtonWalletApiClient::new(test_spec().0, test_spec().1, None, None, None, None).unwrap();
        client.0.login("pro", "pro").await.unwrap();
    }
}
