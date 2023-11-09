use proton_wallet_common::{ DescriptorPublicKey };
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmDescriptorPublicKey {
    inner: DescriptorPublicKey,
}

#[wasm_bindgen]
impl WasmDescriptorPublicKey {

    // pub fn new(inner: DescriptorPublicKey) -> Self {
    //     WasmDescriptorPublicKey { inner }
    // }

    #[wasm_bindgen(constructor)]
    pub fn from_string(public_key: &str) -> Result<WasmDescriptorPublicKey, JsValue> {
        DescriptorPublicKey::from_string(public_key.to_string())
            .map(|pk| WasmDescriptorPublicKey { inner: pk })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // // Method to derive a new WasmDescriptorPublicKey
    // pub fn derive(&self, path: JsValue) -> Result<WasmDescriptorPublicKey, JsValue> {
    //     // Assuming you have a method to convert JsValue to a DerivationPath
    //     let derivation_path: DerivationPath = path.into_serde().map_err(|_| JsValue::from_str("Invalid derivation path"))?;

    //     self.inner.derive(derivation_path)
    //         .map(|pk| WasmDescriptorPublicKey { inner: pk })
    //         .map_err(|e| e.into())
    // }

    // // Method to extend the DescriptorPublicKey
    // pub fn extend(&self, path: JsValue) -> Result<WasmDescriptorPublicKey, JsValue> {
    //     let derivation_path: DerivationPath = path.into_serde().map_err(|_| JsValue::from_str("Invalid derivation path"))?;

    //     self.inner.extend(derivation_path)
    //         .map(|pk| WasmDescriptorPublicKey { inner: pk })
    //         .map_err(|e| e.into())
    // }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}
