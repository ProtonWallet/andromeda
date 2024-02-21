use andromeda_api::network::NetworkClient;
use wasm_bindgen::prelude::*;

use crate::common::{error::WasmError, types::WasmNetwork};

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
    pub async fn get_network(&self) -> Result<WasmNetwork, WasmError> {
        self.0.get_network().await.map(|n| n.into()).map_err(|e| e.into())
    }
}
