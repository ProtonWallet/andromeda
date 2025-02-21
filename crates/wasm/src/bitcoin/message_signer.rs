use andromeda_bitcoin::{message_signer::MessageSigner, SigningType};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::common::{error::ErrorExt, types::WasmScriptType};

use super::account::WasmAccount;

#[wasm_bindgen]
pub struct WasmMessageSigner {
    inner: MessageSigner,
}

impl WasmMessageSigner {
    pub fn get_inner(&self) -> &MessageSigner {
        &self.inner
    }
}

#[wasm_bindgen]
impl WasmMessageSigner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MessageSigner {},
        }
    }

    #[wasm_bindgen(js_name = signMessage)]
    pub async fn sign_message(
        &self,
        account: &WasmAccount,
        message: &str,
        signing_type: WasmSigningType,
        script_type: WasmScriptType,
        btc_address: &str,
    ) -> Result<String, js_sys::Error> {
        Ok(self
            .inner
            .sign_message(
                &account.get_inner(),
                message,
                signing_type.into(),
                script_type.into(),
                btc_address,
            )
            .await
            .map_err(|e| e.to_js_error())?)
    }

    #[wasm_bindgen(js_name = verifyMessage)]
    pub async fn verify_message(
        &self,
        account: &WasmAccount,
        message: &str,
        signature: &str,
        btc_address: &str,
    ) -> Result<(), js_sys::Error> {
        Ok(self
            .inner
            .verify_message(&account.get_inner(), message, signature, btc_address)
            .await
            .map_err(|e| e.to_js_error())?)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum WasmSigningType {
    Electrum = 1,
    Bip137 = 2,
}

impl From<WasmSigningType> for SigningType {
    fn from(wasm_signing_type: WasmSigningType) -> Self {
        match wasm_signing_type {
            WasmSigningType::Electrum => SigningType::Electrum,
            WasmSigningType::Bip137 => SigningType::Bip137,
        }
    }
}
