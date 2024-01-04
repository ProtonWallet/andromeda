use proton_wallet_common::AddressInfo;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub struct WasmAddressInfo {
    inner: AddressInfo,
}

#[wasm_bindgen]
impl WasmAddressInfo {
    #[wasm_bindgen(getter)]
    pub fn index(&self) -> u32 {
        self.inner.index
    }

    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

impl Into<WasmAddressInfo> for AddressInfo {
    fn into(self) -> WasmAddressInfo {
        WasmAddressInfo { inner: self }
    }
}
