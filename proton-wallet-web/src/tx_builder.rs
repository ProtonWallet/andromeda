use wasm_bindgen::prelude::*;
use proton_wallet_common::{TxBuilder, Wallet};

use crate::script::WasmScript;

#[wasm_bindgen]
pub struct WasmTxBuilder {
    inner: TxBuilder,
}

#[wasm_bindgen]
impl WasmTxBuilder {

    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTxBuilder {
        WasmTxBuilder {
            inner: TxBuilder::new(),
        }
    }

    pub fn add_recipient(&mut self, script: WasmScript, amount: u64) -> () {
        // self.inner.add_recipient(script, amount);
        // self.clone()
        //TODO fix this
    }

    pub fn fee_rate(&mut self, sat_per_vbyte: f32) -> () {
        self.inner.fee_rate(sat_per_vbyte);
    }

    // pub fn finish(&self, wallet: Wallet) -> Result<crate::partially_signed_transaction::WasmPartiallySignedTransaction, JsValue> {
    //     self.inner.finish(&wallet)
    //         .map_err(|e| JsValue::from_str(&e.to_string()))
    //         .map(PartiallySignedTransaction::from)
    // }
}
