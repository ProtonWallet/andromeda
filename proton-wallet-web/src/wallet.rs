use wasm_bindgen::prelude::*;
use proton_wallet_common::{BdkWallet, BdkNetwork};

use crate::{descriptor::WasmDescriptor, defined::WasmNetwork};

#[wasm_bindgen]
pub struct WasmWallet {
    inner: BdkWallet,
}

#[wasm_bindgen]
impl WasmWallet {

    #[wasm_bindgen(constructor, js_name = newNoPersist)]
    pub fn new_no_persist(descriptor: WasmDescriptor, change_descriptor: Option<WasmDescriptor>, network: WasmNetwork) -> Result<WasmWallet, JsValue> {
        let descriptor = descriptor.as_string_private();
        let change_descriptor = change_descriptor.map(|d| d.as_string_private());
        let net: BdkNetwork = network.into();
        let wallet: BdkWallet = BdkWallet::new_no_persist(&descriptor, change_descriptor.as_ref(), net).unwrap();

        Ok(WasmWallet {
            inner: wallet,
        })
    }

    // pub fn get_address(&self, address_index: WasmAddressIndex) -> WasmAddressInfo {
    //     let addressIdex = address_index.into();
    //     self.inner.get_address(addressIdex)
    //     // Assuming get_address returns AddressInfo
    // }

    // pub fn get_internal_address(&self, address_index: AddressIndex) -> AddressInfo {
    //     self.inner.get_internal_address(address_index)
    //     // Assuming get_internal_address returns AddressInfo
    // }

    // pub fn network(&self) -> Network {
    //     self.inner.network()
    //     // Assuming network returns Network
    // }

    // pub fn get_balance(&self) -> Balance {
    //     self.inner.get_balance()
    //     // Assuming get_balance returns Balance
    // }

    // pub fn is_mine(&self, script: Script) -> bool {
    //     self.inner.is_mine(script)
    //     // Assuming is_mine returns a bool
    // }

    // pub fn apply_update(&self, update: Update) -> Result<(), JsValue> {
    //     self.inner.apply_update(update)
    //         .map_err(|e| JsValue::from_str(&e.to_string()))
    //     // Assuming apply_update can throw a BdkError
    // }
}
