use proton_wallet_common::mnemonic::{BdkLanguage, Mnemonic};
use wasm_bindgen::prelude::*;

use crate::{error::DetailledWasmError, types::defined::WasmWordCount};

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmLanguage {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Czech,
    French,
    Italian,
    Japanese,
    Korean,
    Spanish,
}

impl From<WasmLanguage> for BdkLanguage {
    fn from(value: WasmLanguage) -> Self {
        match value {
            _ => BdkLanguage::English,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmMnemonic {
    inner: Mnemonic,
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
    pub fn from_string(mnemonic: &str) -> Result<WasmMnemonic, DetailledWasmError> {
        Mnemonic::from_string(mnemonic.to_string())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic.into() })
            .map_err(|e| e.into())
    }

    /// Create a new Mnemonic from the given entropy.
    #[wasm_bindgen(js_name = fromEntropy)]
    pub fn from_entropy(entropy: &[u8]) -> Result<WasmMnemonic, DetailledWasmError> {
        Mnemonic::from_entropy(entropy.to_vec())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic.into() })
            .map_err(|e| e.into())
    }

    /// Returns the Mnemonic as a string.
    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }

    // Returns the mnemonic as words array
    #[wasm_bindgen(js_name = toWords)]
    pub fn to_words(&self) -> Vec<String> {
        self.inner.to_words()
    }
}
