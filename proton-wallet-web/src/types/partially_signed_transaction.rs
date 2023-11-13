
use wasm_bindgen::prelude::*;
use proton_wallet_common::PartiallySignedTransaction;
use super::transaction::WasmBdkTransaction;

#[wasm_bindgen]
pub struct WasmPartiallySignedTransaction {
    inner: PartiallySignedTransaction,
}

#[wasm_bindgen]
impl WasmPartiallySignedTransaction {
    
    #[wasm_bindgen(constructor)]
    pub fn new(psbt_base64: &str) -> Result<WasmPartiallySignedTransaction, JsValue> {
        let psbt = PartiallySignedTransaction::new(psbt_base64.to_string())
            .map_err(|e| JsValue::from_str(&format!("WasmPartiallySignedTransaction error: {}", e)))?;
        Ok(WasmPartiallySignedTransaction { inner: psbt })
    }

    pub fn serialize(&self) -> String {
        self.inner.serialize()
    }

    pub fn extract_tx(&self) -> Result<WasmBdkTransaction, JsValue> {
        let arc_tx = self.inner.extract_tx();
        Ok((*arc_tx).clone().into())
    }
}