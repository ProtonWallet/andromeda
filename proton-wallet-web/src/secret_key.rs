use crate::public_key::WasmDescriptorPublicKey;
use proton_wallet_common::DescriptorSecretKey;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmDescriptorSecretKey {
    inner: Arc<DescriptorSecretKey>,
}

#[wasm_bindgen]
impl WasmDescriptorSecretKey {
    // #[wasm_bindgen(constructor)]
    // pub fn new(
    //     network: WasmNetwork,
    //     mnemonic: WasmMnemonic,
    //     password: Option<String>,
    // ) -> Result<WasmDescriptorSecretKey, JsValue> {
    //     let mnemonic = mnemonic.get_inner();
    //     let mnemonic = Mnemonic::from_js_value(mnemonic.get_inner()).map_err(|e| JsValue::from_str(&e.to_string()))?;
    //     let mnemonic_arc = Arc::new(mnemonic); // Wrap the mnemonic in an Arc for shared ownership.
    //     let secret_key = DescriptorSecretKey::new(network.into(), mnemonic_arc, password);
    //     Ok(WasmDescriptorSecretKey {
    //         inner: Arc::new(secret_key),
    //     })
    // }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(secret_key_str: &str) -> Result<WasmDescriptorSecretKey, JsValue> {
        DescriptorSecretKey::from_string(secret_key_str.to_string())
            .map(|secret_key| WasmDescriptorSecretKey {
                inner: Arc::new(secret_key),
            })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // pub fn derive(&self, path: WasmDerivationPath) -> Result<WasmDescriptorSecretKey, JsValue> {
    //     let path = Arc::new(path); // Wrap the path in an Arc for shared ownership.
    //     self.inner.derive(path)
    //         .map(|secret_key| WasmDescriptorSecretKey { inner: secret_key })
    //         .map_err(|e| JsValue::from_str(&e.to_string()))
    // }

    // pub fn extend(&self, path: WasmDerivationPath) -> Result<WasmDescriptorSecretKey, JsValue> {
    //     let path = Arc::new(path); // Wrap the path in an Arc for shared ownership.
    //     self.inner.extend(path)
    //         .map(|secret_key| WasmDescriptorSecretKey { inner: secret_key })
    //         .map_err(|e| JsValue::from_str(&e.to_string()))
    // }

    // This method should convert the secret key to a public key and return it
    pub fn as_public(&self) -> Result<WasmDescriptorPublicKey, JsValue> {
        let public_key = Arc::clone(&self.inner).as_public(); // Clone the Arc to get the public key
        let public_key_str = public_key.as_string(); // Convert the public key to a string
        WasmDescriptorPublicKey::from_string(&public_key_str).map_err(|e| e)
    }

    pub fn secret_bytes(&self) -> Vec<u8> {
        self.inner.secret_bytes()
    }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }

    // pub fn get_inner(&self) -> JsValue {
    //     // Serialize the Mnemonic to a JsValue
    //     match serde_wasm_bindgen::to_value(&self.inner) {
    //         Ok(js_value) => js_value,
    //         Err(_) => JsValue::UNDEFINED, // or handle the error as you see fit
    //     }
    // }
}
