use std::str::FromStr;

use andromeda_api::wallet::FromParts;
use andromeda_bitcoin::DerivationPath;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::typescript_interfaces::IWasmDerivationPath;
use crate::common::{
    error::{DetailledWasmError, WasmError},
    types::WasmNetwork,
};

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmDerivationPath {
    inner: DerivationPath,
}

#[wasm_bindgen]
impl WasmDerivationPath {
    #[wasm_bindgen(constructor)]
    pub fn new(path: &str) -> Result<WasmDerivationPath, DetailledWasmError> {
        let derivation_path = DerivationPath::from_str(path).map_err(|_| WasmError::InvalidDerivationPath.into())?;

        Ok(WasmDerivationPath { inner: derivation_path })
    }

    #[wasm_bindgen(js_name = fromRawTs)]
    pub fn from_raw_ts(raw_ts: IWasmDerivationPath) -> WasmDerivationPath {
        serde_wasm_bindgen::from_value(raw_ts.into()).unwrap()
    }

    #[wasm_bindgen(js_name = fromParts)]
    pub fn from_parts(purpose: u32, network: WasmNetwork, account_index: u32) -> WasmDerivationPath {
        Self {
            inner: DerivationPath::from_parts(purpose, network.into(), account_index),
        }
    }
}

impl Into<DerivationPath> for &WasmDerivationPath {
    fn into(self) -> DerivationPath {
        self.inner.clone()
    }
}

impl Into<WasmDerivationPath> for DerivationPath {
    fn into(self) -> WasmDerivationPath {
        WasmDerivationPath { inner: self.clone() }
    }
}
