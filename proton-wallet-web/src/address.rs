use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test_address() -> String {
    return proton_wallet_common::address::test_address();
}