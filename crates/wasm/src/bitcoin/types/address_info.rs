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

impl From<AddressInfo> for WasmAddressInfo {
    fn from(value: AddressInfo) -> Self {
        WasmAddressInfo {
            index: value.index,
            address: value.address.to_string(),
            keychain: value.keychain.into(),
        }
    }
}
