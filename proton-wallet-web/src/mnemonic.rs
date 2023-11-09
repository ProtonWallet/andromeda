use crate::defined::WasmWordCount;
use proton_wallet_common::mnemonic::{BdkLanguage, BdkMnemonic, Mnemonic};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmBdkMnemonic {
    lang: WasmLanguage,
    words: String,
}

impl From<WasmBdkMnemonic> for BdkMnemonic {
    fn from(value: WasmBdkMnemonic) -> Self {
        BdkMnemonic::parse_in(value.lang.into(), value.words).unwrap()
    }
}

impl Into<WasmBdkMnemonic> for Mnemonic {
    fn into(self) -> WasmBdkMnemonic {
        WasmBdkMnemonic {
            lang: WasmLanguage::English,
            words: self.inner().word_iter().collect::<Vec<&str>>().join(" "),
        }
    }
}

#[wasm_bindgen]
pub struct WasmMnemonic {
    inner: WasmBdkMnemonic, // This is the original Mnemonic struct from your Rust code.
}

#[wasm_bindgen]
impl WasmMnemonic {
    /// Generates a Mnemonic with a random entropy based on the given word count.
    #[wasm_bindgen(constructor)]
    pub fn new(word_count: WasmWordCount) -> Result<WasmMnemonic, JsValue> {
        let mnemonic = Mnemonic::new(word_count.into());

        Ok(WasmMnemonic {
            inner: WasmBdkMnemonic {
                lang: WasmLanguage::English,
                words: mnemonic.as_string(),
            },
        })
    }

    /// Parse a Mnemonic with the given string.
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(mnemonic: &str) -> Result<WasmMnemonic, JsValue> {
        Mnemonic::from_string(mnemonic.to_string())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic.into() })
            .map_err(|e| JsValue::from_str(&e.to_string())) // TODO: use Error enum
    }

    /// Create a new Mnemonic from the given entropy.
    #[wasm_bindgen(js_name = fromEntropy)]
    pub fn from_entropy(entropy: &[u8]) -> Result<WasmMnemonic, JsValue> {
        Mnemonic::from_entropy(entropy.to_vec())
            .map(|mnemonic| WasmMnemonic { inner: mnemonic.into() })
            .map_err(|e| JsValue::from_str(&e.to_string())) // TODO: use Error enum
    }

    /// Returns the Mnemonic as a string.
    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        BdkMnemonic::from(self.inner()).to_string()
    }

    #[wasm_bindgen]
    pub fn inner(&self) -> WasmBdkMnemonic {
        self.inner.clone()
    }
}
