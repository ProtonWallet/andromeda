use andromeda_bitcoin::WordCount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum WasmWordCount {
    Words12,
    Words15,
    Words18,
    Words21,
    Words24,
}

impl From<WasmWordCount> for WordCount {
    fn from(value: WasmWordCount) -> Self {
        match value {
            WasmWordCount::Words12 => WordCount::Words12,
            WasmWordCount::Words15 => WordCount::Words15,
            WasmWordCount::Words18 => WordCount::Words18,
            WasmWordCount::Words21 => WordCount::Words21,
            WasmWordCount::Words24 => WordCount::Words24,
        }
    }
}
