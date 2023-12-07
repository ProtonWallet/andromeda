use proton_wallet_common::client::Client;
use wasm_bindgen::prelude::*;

use crate::{
    error::{DetailledWasmError, WasmError},
    psbt::WasmPartiallySignedTransaction,
};

#[wasm_bindgen(getter_with_clone)]
pub struct WasmClient {
    inner: Client,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

impl Into<Client> for WasmClient {
    fn into(self) -> Client {
        self.inner
    }
}

#[wasm_bindgen]
impl WasmClient {
    /// Generates a Mnemonic with a random entropy based on the given word count.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmClient, DetailledWasmError> {
        let inner = Client::new(None).map_err(|e| e.into())?;

        Ok(WasmClient { inner })
    }

    #[wasm_bindgen]
    pub async fn get_fees_estimation(&self) -> FeeRateByBlockEstimation {
        let fees_estimation = self.inner.get_fees_estimation().await.unwrap_or_default();
        serde_wasm_bindgen::to_value(&fees_estimation).unwrap().into()
    }

    #[wasm_bindgen]
    pub async fn broadcast_psbt(&self, psbt: WasmPartiallySignedTransaction) -> Result<String, WasmError> {
        let tx = psbt.get_inner().extract_tx();

        self.inner
            .broadcast(tx.clone())
            .await
            .map_err(|_| WasmError::ChecksumMismatch)?;

        Ok(tx.txid().to_string())
    }
}
