use std::str::FromStr;

use proton_wallet_common::DerivationPath;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::error::{DetailledWasmError, WasmError};

use super::typescript_interfaces::IWasmDerivationPath;

#[wasm_bindgen]
#[derive(Clone, Serialize)]
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
        WasmDerivationPath {
            inner: serde_wasm_bindgen::from_value(raw_ts.into()).unwrap(),
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
