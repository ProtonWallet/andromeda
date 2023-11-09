use std::sync::Arc;

use wasm_bindgen::prelude::*;
// use bdk::bitcoin::Network;
use proton_wallet_common::Descriptor;

use crate::{defined::{WasmNetwork, WasmKeychainKind}, secret_key::WasmDescriptorSecretKey};
// use bdk::Error as BdkError;
// use std::sync::Arc;
// use bdk::keys::{DescriptorSecretKey as BdkDescriptorSecretKey, DescriptorPublicKey as BdkDescriptorPublicKey};
// use bdk::wallet::KeychainKind;
// // ... other necessary imports ...

#[wasm_bindgen]
pub struct WasmDescriptor {
    inner: Descriptor,
}

#[wasm_bindgen]
impl WasmDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(descriptor: &str, network: WasmNetwork) -> Result<WasmDescriptor, JsValue> {
        Descriptor::new(descriptor.to_string(), network.into())
            .map(|descriptor| WasmDescriptor { inner: descriptor })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // BIP44 constructor for secret keys
    // #[wasm_bindgen(js_name = newBip44)]
    // pub fn new_bip44(secret_key: WasmDescriptorSecretKey, keychain_kind: WasmKeychainKind, network: WasmNetwork) -> Result<WasmDescriptor, JsValue> {
    //     let secret_key = Arc::new(secret_key.get_inner()?); // This method needs to be implemented
    //     Ok(WasmDescriptor {
    //         inner: Descriptor::new_bip44(secret_key, keychain_kind.into(), network.into()),
    //     })
    // }

    // Add other constructors for public keys, BIP49, BIP84, and BIP86 accordingly...

    // Method to get the descriptor as a string
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }

    // Method to get the private descriptor as a string
    #[wasm_bindgen(js_name = asStringPrivate)]
    pub fn as_string_private(&self) -> String {
        self.inner.as_string_private()
    }
}