use proton_wallet_common::AddressIndex;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub struct WasmAddressIndex {
    inner: AddressIndex,
}

//TODO add from <>
#[wasm_bindgen]
impl WasmAddressIndex {
    #[wasm_bindgen(js_name = createNew)]
    pub fn new() -> WasmAddressIndex {
        WasmAddressIndex {
            inner: AddressIndex::New,
        }
    }

    #[wasm_bindgen(js_name = createLastUnused)]
    pub fn last_unused() -> WasmAddressIndex {
        WasmAddressIndex {
            inner: AddressIndex::LastUnused,
        }
    }

    #[wasm_bindgen(js_name = createPeek)]
    pub fn peek(index: u32) -> WasmAddressIndex {
        WasmAddressIndex {
            inner: AddressIndex::Peek { index },

        }
    }
}
