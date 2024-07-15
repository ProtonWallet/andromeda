use andromeda_api::email_integration::{ApiWalletBitcoinAddressLookup, EmailIntegrationClient};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletBitcoinAddressLookup {
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
}

impl From<ApiWalletBitcoinAddressLookup> for WasmApiWalletBitcoinAddressLookup {
    fn from(value: ApiWalletBitcoinAddressLookup) -> Self {
        Self {
            BitcoinAddress: value.BitcoinAddress,
            BitcoinAddressSignature: value.BitcoinAddressSignature,
        }
    }
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmApiWalletBitcoinAddressLookupData {
    pub Data: WasmApiWalletBitcoinAddressLookup,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmEmailIntegrationClient(EmailIntegrationClient);

impl From<EmailIntegrationClient> for WasmEmailIntegrationClient {
    fn from(value: EmailIntegrationClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmEmailIntegrationClient {
    #[wasm_bindgen(js_name = "lookupBitcoinAddress")]
    pub async fn lookup_bitcoin_address(
        &self,
        email: String,
    ) -> Result<WasmApiWalletBitcoinAddressLookupData, JsValue> {
        self.0
            .lookup_bitcoin_address(email)
            .await
            .map(|lookup| WasmApiWalletBitcoinAddressLookupData { Data: lookup.into() })
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "createBitcoinAddressesRequest")]
    pub async fn create_bitcoin_addresses_request(&self, email: String) -> Result<(), JsValue> {
        self.0
            .create_bitcoin_addresses_request(email)
            .await
            .map_err(|e| e.to_js_error())
    }
}
