use andromeda_api::{self, AccessToken, AuthData, ProtonWalletApiClient, RefreshToken, Scope, Uid};
use wasm_bindgen::prelude::*;

use crate::common::error::WasmError;
use network::WasmNetworkClient;
use settings::WasmSettingsClient;
use wallet::WasmWalletClient;

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

impl Into<AuthData> for WasmAuthData {
    fn into(self) -> AuthData {
        AuthData {
            uid: serde_json::from_str::<Uid>(&self.uid).unwrap(),
            access: serde_json::from_str::<AccessToken>(&self.access).unwrap(),
            refresh: serde_json::from_str::<RefreshToken>(&self.refresh).unwrap(),
            scopes: serde_json::from_str::<Vec<Scope>>(&self.uid).unwrap(),
        }
    }
}

#[wasm_bindgen]
pub struct WasmProtonWalletApiClient(ProtonWalletApiClient);

#[wasm_bindgen]
impl WasmProtonWalletApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(auth: Option<WasmAuthData>) -> Result<WasmProtonWalletApiClient, WasmError> {
        let client = if let Some(auth) = auth {
            ProtonWalletApiClient::from_auth(auth.into()).map_err(|e| e.into())?
        } else {
            ProtonWalletApiClient::default()
        };

        Ok(WasmProtonWalletApiClient(client))
    }

    #[wasm_bindgen]
    pub async fn login(&mut self) -> Result<(), WasmError> {
        self.0.login("pro", "pro").await.map_err(|e| e.into())
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
        let mut client = WasmProtonWalletApiClient::new(None).unwrap();
        client.0.login("pro", "pro").await.unwrap();
    }
}
