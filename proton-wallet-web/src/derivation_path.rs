use std::sync::Mutex;

use proton_wallet_common::DerivationPath;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmDerivationPath {
    inner_mutex: Mutex<DerivationPath>,
}

#[wasm_bindgen]
impl WasmDerivationPath {
    #[wasm_bindgen(constructor)]
    pub fn new(path: &str) -> Result<WasmDerivationPath, JsValue> {
        let derivation_path = DerivationPath::new(path.to_string()).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmDerivationPath {
            inner_mutex: Mutex::new(derivation_path),
        })
    }
}
