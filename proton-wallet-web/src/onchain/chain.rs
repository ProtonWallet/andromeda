use proton_wallet_common::common::chain::Chain;
use wasm_bindgen::prelude::*;

use super::{account::WasmAccount, psbt::WasmPartiallySignedTransaction};
use crate::common::error::{DetailledWasmError, WasmError};

#[wasm_bindgen(getter_with_clone)]
pub struct WasmChain {
    inner: Chain,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

impl Into<Chain> for WasmChain {
    fn into(self) -> Chain {
        self.inner
    }
}

#[wasm_bindgen]
impl WasmChain {
    /// Generates a Mnemonic with a random entropy based on the given word count.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmChain, DetailledWasmError> {
        let inner = Chain::new(None).map_err(|e| e.into())?;
        Ok(WasmChain { inner })
    }

    #[wasm_bindgen(js_name = getFeesEstimation)]
    pub async fn get_fees_estimation(&mut self) -> FeeRateByBlockEstimation {
        let fees_estimation = self.inner.get_fees_estimation().await.unwrap_or_default();
        serde_wasm_bindgen::to_value(&fees_estimation).unwrap().into()
    }

    #[wasm_bindgen(js_name = fullSync)]
    pub async fn full_sync(&self, account: &WasmAccount) -> Result<(), DetailledWasmError> {
        let account_inner = account.get_inner();

        self.inner
            .full_sync(
                account_inner
                    .write()
                    .await
                    .map_err(|_| WasmError::LockError.into())?
                    .get_mutable_wallet(),
            )
            .await
            .map_err(|e| e.into())?;

        account_inner.release_write_lock();
        Ok(())
    }
    #[wasm_bindgen(js_name = partialSync)]
    pub async fn partial_sync(&self, account: &WasmAccount) -> Result<(), DetailledWasmError> {
        let account_inner = account.get_inner();

        self.inner
            .partial_sync(
                account_inner
                    .write()
                    .await
                    .map_err(|_| WasmError::LockError.into())?
                    .get_mutable_wallet(),
            )
            .await
            .map_err(|e| e.into())?;

        account_inner.release_write_lock();
        Ok(())
    }

    #[wasm_bindgen(js_name = broadcastPsbt)]
    pub async fn broadcast_psbt(&self, psbt: &WasmPartiallySignedTransaction) -> Result<String, WasmError> {
        let tx = psbt.get_inner().extract_tx();

        self.inner
            .broadcast(tx.clone())
            .await
            .map_err(|_| WasmError::ChecksumMismatch)?;

        Ok(tx.txid().to_string())
    }
}
