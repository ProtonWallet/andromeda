use andromeda_bitcoin::account_syncer::AccountSyncer;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::common::error::ErrorExt;

use super::{account::WasmAccount, blockchain_client::WasmBlockchainClient};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAccountSyncer {
    inner: AccountSyncer,
}

#[wasm_bindgen]
impl WasmAccountSyncer {
    #[wasm_bindgen(constructor)]
    pub fn new(client: &WasmBlockchainClient, account: &WasmAccount) -> Self {
        Self {
            inner: AccountSyncer::new(client.into(), account.get_inner()),
        }
    }
}

#[wasm_bindgen]
impl WasmAccountSyncer {
    #[wasm_bindgen(js_name = fullSync)]
    pub async fn full_sync(&self, stop_gap: Option<usize>) -> Result<(), js_sys::Error> {
        Ok(self.inner.full_sync(stop_gap).await.map_err(|e| e.to_js_error())?)
    }

    #[wasm_bindgen(js_name = partialSync)]
    pub async fn partial_sync(&self) -> Result<(), js_sys::Error> {
        Ok(self.inner.partial_sync().await.map_err(|e| e.to_js_error())?)
    }

    #[wasm_bindgen(js_name = shouldSync)]
    pub async fn should_sync(&self) -> Result<bool, js_sys::Error> {
        Ok(self.inner.should_sync().await.map_err(|e| e.to_js_error())?)
    }
}
