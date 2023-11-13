use wasm_bindgen::prelude::*;
use proton_wallet_common::AddressInfo;

#[wasm_bindgen]
pub struct WasmAddressInfo {
    inner: AddressInfo,
}

//TODO re expose WasmBdkAddressInfo
#[wasm_bindgen]
impl WasmAddressInfo {
    // #[wasm_bindgen(constructor)]
    // pub fn new(index: u32, address: WasmAddress, keychain: WasmKeychainKind) -> WasmAddressInfo {
    //     let wasm_kc: KeychainKind = keychain.into();
    //     let arc_addinfo: Arc<Address> = Arc::new(address.into());
    //     WasmAddressInfo { inner: AddressInfo { index, arc_addinfo, wasm_kc } }
    // }

    #[wasm_bindgen(getter)]
    pub fn index(&self) -> u32 {
        self.inner.index
    }

    // #[wasm_bindgen(getter)]
    // pub fn address(&self) -> WasmAddress {
    //     self.inner.address.clone()
    // }

    // #[wasm_bindgen(getter)]
    // pub fn keychain(&self) -> WasmKeychainKind {
    //     self.inner.keychain // Assuming KeychainKind is wasm_bindgen compatible
    // }
}
