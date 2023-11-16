use std::str::FromStr;

use proton_wallet_common::DerivationPath;
use wasm_bindgen::prelude::*;

use crate::error::{DetailledWasmError, WasmError};

#[wasm_bindgen]
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
}

impl Into<DerivationPath> for WasmDerivationPath {
    fn into(self) -> DerivationPath {
        self.inner
    }
}
