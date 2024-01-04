use std::str::FromStr;

use proton_wallet_common::{Address, ScriptBuf};
use wasm_bindgen::prelude::*;

use crate::common::error::WasmError;

use super::{defined::WasmNetwork, transaction::WasmScript};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAddress {
    inner: Address,
}

impl Into<Address> for &WasmAddress {
    fn into(self) -> Address {
        let addr = self.inner.clone();
        addr
    }
}

impl Into<WasmAddress> for Address {
    fn into(self) -> WasmAddress {
        WasmAddress { inner: self.clone() }
    }
}

#[wasm_bindgen]
impl WasmAddress {
    #[wasm_bindgen(constructor)]
    pub fn new(str: String, network: WasmNetwork) -> Result<WasmAddress, WasmError> {
        let inner = Address::from_str(&str)
            .map_err(|_| WasmError::InvalidAddress)?
            .require_network(network.into())
            .map_err(|_| WasmError::InvalidNetwork)?;

        Ok(WasmAddress { inner })
    }

    #[wasm_bindgen(js_name = fromScript)]
    pub fn from_script(value: WasmScript, network: WasmNetwork) -> Result<WasmAddress, WasmError> {
        let script: ScriptBuf = value.into();

        Ok(WasmAddress {
            inner: Address::from_script(&script, network.into()).map_err(|_| WasmError::InvalidAddress)?,
        })
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = intoScript)]
    pub fn into_script(&self) -> WasmScript {
        self.inner.script_pubkey().into()
    }
}
