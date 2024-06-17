use andromeda_api::{self, ApiConfig, Auth, ProtonWalletApiClient};
use bitcoin_address::WasmBitcoinAddressClient;
use email_integration::WasmEmailIntegrationClient;
use exchange_rate::WasmExchangeRateClient;
use network::WasmNetworkClient;
use payment_gateway::WasmPaymentGatewayClient;
use settings::WasmSettingsClient;
use wallet::WasmWalletClient;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

mod bitcoin_address;
mod email_integration;
mod env;
mod exchange_rate;
mod network;
mod payment_gateway;
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
    pub settings: WasmSettingsClient,
    pub network: WasmNetworkClient,
    pub wallet: WasmWalletClient,
}

#[wasm_bindgen]
impl WasmProtonWalletApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        uid_str: Option<String>,
        origin: Option<String>,
        url_prefix: Option<String>,
    ) -> Result<WasmProtonWalletApiClient, js_sys::Error> {
        let config = ApiConfig {
            // TODO: add clients specs here
            spec: None,
            auth: uid_str.map(|u| Auth::external(u)),
            env: origin,
            url_prefix,
            store: None,
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
            settings: WasmSettingsClient::from(clients.settings),
            network: WasmNetworkClient::from(clients.network),
            wallet: WasmWalletClient::from(clients.wallet),
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::WasmProtonWalletApiClient;

    #[wasm_bindgen_test]
    #[ignore]
    async fn should_create_pw_api_client() {
        let client = WasmProtonWalletApiClient::new(None, None, None).unwrap();
        client.0.login("pro", "pro").await.unwrap();
    }
}
