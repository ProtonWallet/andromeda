use crate::defined::WasmWordCount;
use proton_wallet_common::Mnemonic;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmMnemonic {
    inner: Mnemonic, // This is the original Mnemonic struct from your Rust code.
}

#[wasm_bindgen]
impl WasmMnemonic {
    /// Generates a Mnemonic with a random entropy based on the given word count.
    #[wasm_bindgen(constructor)]
    pub fn new(word_count: WasmWordCount) -> Result<WasmMnemonic, JsValue> {
        let mnemonic = Mnemonic::new(word_count.into());
        Ok(WasmMnemonic { inner: mnemonic })
    }

    /// Parse a Mnemonic with the given string.
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(mnemonic: &str) -> Result<WasmMnemonic, JsValue> {
        Mnemonic::from_string(mnemonic.to_string())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create a new Mnemonic from the given entropy.
    #[wasm_bindgen(js_name = fromEntropy)]
    pub fn from_entropy(entropy: &[u8]) -> Result<WasmMnemonic, JsValue> {
        Mnemonic::from_entropy(entropy.to_vec())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Returns the Mnemonic as a string.
    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }

    pub fn get_inner(&self) -> JsValue {
        // Serialize the Mnemonic to a JsValue
        match serde_wasm_bindgen::to_value(&self.inner) {
            Ok(js_value) => js_value,
            Err(_) => JsValue::UNDEFINED, // or handle the error as you see fit
        }
    }
}
