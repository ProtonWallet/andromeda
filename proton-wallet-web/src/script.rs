use wasm_bindgen::prelude::*;
use proton_wallet_common::Script;

#[wasm_bindgen]
pub struct WasmScript {
    inner: Script,
}

#[wasm_bindgen]
impl WasmScript {
    #[wasm_bindgen(constructor)]
    pub fn new(raw_output_script: Vec<u8>) -> WasmScript {
        WasmScript { inner: Script::new(raw_output_script).into() }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes()
    }
}