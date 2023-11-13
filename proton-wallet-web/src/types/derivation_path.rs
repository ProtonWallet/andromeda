use proton_wallet_common::keys::DerivationPath;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

use crate::error::WasmError;

#[wasm_bindgen]
pub struct WasmDerivationPath {
    inner: DerivationPath,
}

#[wasm_bindgen]
impl WasmDerivationPath {
    #[wasm_bindgen(constructor)]
    pub fn new(path: &str) -> Result<WasmDerivationPath, WasmError> {
        let derivation_path = DerivationPath::from_str(path).map_err(|_| WasmError::InvalidDerivationPath)?;

        Ok(WasmDerivationPath { inner: derivation_path })
    }
}

impl Into<DerivationPath> for WasmDerivationPath {
    fn into(self) -> DerivationPath {
        self.inner
    }
}
