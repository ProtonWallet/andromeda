use andromeda_api::{self, ApiConfig, AuthData, ProtonWalletApiClient, Uid};
use exchange_rate::WasmExchangeRateClient;
use network::WasmNetworkClient;
use settings::WasmSettingsClient;
use wallet::WasmWalletClient;
use wasm_bindgen::prelude::*;

use crate::{api::env::BrowserOriginEnv, common::error::WasmError};

mod env;
mod exchange_rate;
mod network;
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
pub struct WasmProtonWalletApiClient(ProtonWalletApiClient);

#[wasm_bindgen]
impl WasmProtonWalletApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(uid_str: Option<String>, origin: Option<String>) -> Result<WasmProtonWalletApiClient, WasmError> {
        let config = ApiConfig {
            // TODO: add clients specs here
            spec: None,
            auth: uid_str.map(|u| AuthData::Uid(Uid::from(u))),
            env: origin.map(|o| BrowserOriginEnv::new(o)),
        };

        let client = ProtonWalletApiClient::from_config(config);

        Ok(WasmProtonWalletApiClient(client))
    }

    /// Returns a client to use exchange rate API
    #[wasm_bindgen]
    pub fn exchange_rate(&self) -> WasmExchangeRateClient {
        WasmExchangeRateClient::from(self.0.exchange_rate.clone())
    }

    /// Returns a client to use settings API
    #[wasm_bindgen]
    pub fn settings(&self) -> WasmSettingsClient {
        WasmSettingsClient::from(self.0.settings.clone())
    }

    /// Returns a client to use network API
    #[wasm_bindgen]
    pub fn network(&self) -> WasmNetworkClient {
        WasmNetworkClient::from(self.0.network.clone())
    }

    /// Returns a client to use wallet API
    #[wasm_bindgen]
    pub fn wallet(&self) -> WasmWalletClient {
        WasmWalletClient::from(self.0.wallet.clone())
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::WasmProtonWalletApiClient;

    #[wasm_bindgen_test]
    #[ignore]
    async fn should_create_pw_api_client() {
        let mut client = WasmProtonWalletApiClient::new(None, None).unwrap();
        client.0.login("pro", "pro").await.unwrap();
    }
}
