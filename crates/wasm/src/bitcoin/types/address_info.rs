use andromeda_bitcoin::AddressInfo;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::types::WasmKeychainKind;

#[wasm_bindgen(getter_with_clone)]
#[derive(Tsify, Clone, Serialize, Deserialize)]
pub struct WasmAddressInfo {
    /// Child index of this address
    pub index: u32,
    /// Address
    pub address: String,
    /// Type of keychain
    pub keychain: WasmKeychainKind,
}

impl Into<WasmAddressInfo> for AddressInfo {
    fn into(self) -> WasmAddressInfo {
        WasmAddressInfo {
            index: self.index,
            address: self.address.to_string(),
            keychain: self.keychain.into(),
        }
    }
}
