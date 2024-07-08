use std::str::FromStr;

use andromeda_bitcoin::{error::Error as BitcoinError, DerivationPath};
use andromeda_common::FromParts;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::common::{
    error::ErrorExt,
    types::{WasmNetwork, WasmScriptType},
};

#[wasm_bindgen]
#[derive(Clone, Deserialize, Serialize)]
pub struct WasmDerivationPath {
    inner: DerivationPath,
}

#[wasm_bindgen]
impl WasmDerivationPath {
    #[wasm_bindgen(constructor)]
    pub fn new(path: &str) -> Result<WasmDerivationPath, js_sys::Error> {
        let derivation_path = DerivationPath::from_str(path).map_err(|e| BitcoinError::from(e).to_js_error())?;

        Ok(WasmDerivationPath { inner: derivation_path })
    }

    #[wasm_bindgen(js_name = fromParts)]
    pub fn from_parts(script_type: WasmScriptType, network: WasmNetwork, account_index: u32) -> WasmDerivationPath {
        Self {
            inner: DerivationPath::from_parts(script_type.into(), network.into(), account_index),
        }
    }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_str(str: String) -> Result<WasmDerivationPath, js_sys::Error> {
        Ok(Self {
            inner: DerivationPath::from_str(&str).map_err(|e| BitcoinError::from(e).to_js_error())?,
        })
    }
}

impl WasmDerivationPath {
    pub fn inner(&self) -> &DerivationPath {
        &self.inner
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
