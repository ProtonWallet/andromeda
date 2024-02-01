use andromeda_bitcoin::chain::Chain;
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

        let read_lock = account_inner.read().map_err(|_| WasmError::LockError.into())?;
        let (graph_update, chain_update, last_active_indices) = self
            .inner
            .full_sync(read_lock.get_wallet())
            .await
            .map_err(|e| e.into())?;
        drop(read_lock);

        let mut write_lock = account_inner.write().map_err(|_| WasmError::LockError.into())?;
        self.inner
            .commit_sync(
                write_lock.get_mutable_wallet(),
                graph_update,
                chain_update,
                Some(last_active_indices),
            )
            .map_err(|e| e.into())?;

        Ok(())
    }
    #[wasm_bindgen(js_name = partialSync)]
    pub async fn partial_sync(&self, account: &WasmAccount) -> Result<(), DetailledWasmError> {
        let account_inner = account.get_inner();

        let read_lock = account_inner.read().map_err(|_| WasmError::LockError.into())?;
        let (graph_update, chain_update) = self
            .inner
            .partial_sync(read_lock.get_wallet())
            .await
            .map_err(|e| e.into())?;
        drop(read_lock);

        let mut write_lock = account_inner.write().map_err(|_| WasmError::LockError.into())?;

        self.inner
            .commit_sync(write_lock.get_mutable_wallet(), graph_update, chain_update, None)
            .map_err(|e| e.into())?;

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
