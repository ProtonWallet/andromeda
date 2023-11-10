use wasm_bindgen::prelude::*;
use crate::defined::WasmNetwork;
use proton_wallet_common::Address;

#[wasm_bindgen]
pub struct WasmAddress {
    inner: Address,
}

#[wasm_bindgen]
impl WasmAddress {
    #[wasm_bindgen(constructor)]
    pub fn new(address: String, network: WasmNetwork) -> Result<WasmAddress, JsValue> {
        Address::new(address, network.into())
            .map(|addr| WasmAddress { inner: addr })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn network(&self) -> WasmNetwork {
        self.inner.network().into()
    }

    // pub fn script_pubkey(&self) -> Arc<Script> {
    //     Arc::new(Script(self.inner.script_pubkey()))
    // }

    pub fn to_qr_uri(&self) -> String {
        self.inner.to_qr_uri()
    }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}