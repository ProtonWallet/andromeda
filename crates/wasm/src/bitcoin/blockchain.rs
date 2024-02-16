use andromeda_bitcoin::blockchain::Blockchain;
use wasm_bindgen::prelude::*;

use super::{account::WasmAccount, psbt::WasmPartiallySignedTransaction};
use crate::common::error::WasmError;

#[wasm_bindgen(getter_with_clone)]
pub struct WasmBlockchain(Blockchain);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

#[wasm_bindgen]
impl WasmBlockchain {
    #[wasm_bindgen(constructor)]
    pub fn new(url: Option<String>, stop_gap: Option<usize>) -> Self {
        let blockchain = Blockchain::new(url, stop_gap);
        Self(blockchain)
    }

    /// Perform a full sync for the account
    #[wasm_bindgen(js_name = fullSync)]
    pub async fn full_sync(&self, account: &WasmAccount) -> Result<(), WasmError> {
        let inner = account.get_inner();
        let inner = inner.read().unwrap();

        self.0.full_sync(inner.get_wallet()).await.map_err(|e| e.into())?;

        Ok(())
    }

    /// Returns fee estimations in a Map
    #[wasm_bindgen(js_name = getFeesEstimation)]
    pub async fn get_fees_estimation(&self) -> FeeRateByBlockEstimation {
        let fees = self.0.get_fees_estimation().await.unwrap_or_default();
        serde_wasm_bindgen::to_value(&fees).unwrap().into()
    }

    /// Broadcasts a provided transaction
    #[wasm_bindgen(js_name = broadcastPsbt)]
    pub async fn broadcast_psbt(&self, psbt: &WasmPartiallySignedTransaction) -> Result<String, WasmError> {
        let tx = psbt.get_inner().extract_tx();

        self.0.broadcast(tx.clone()).await.map_err(|e| e.into())?;

        Ok(tx.txid().to_string())
    }
}
