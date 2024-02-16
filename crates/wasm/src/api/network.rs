use andromeda_api::network::NetworkClient;
use wasm_bindgen::prelude::*;

use crate::common::error::WasmError;

#[wasm_bindgen]
pub struct WasmNetworkClient(NetworkClient);

impl From<NetworkClient> for WasmNetworkClient {
    fn from(value: NetworkClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmNetworkClient {
    #[wasm_bindgen(js_name = "getNetwork")]
    pub async fn get_network(&self) -> Result<u8, WasmError> {
        self.0.get_network().await.map_err(|e| e.into())
    }
}
