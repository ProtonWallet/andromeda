use std::str::FromStr;

use andromeda_bitcoin::{Address, ScriptBuf};
use serde::{Deserialize, Deserializer, Serialize};
use wasm_bindgen::prelude::*;

use super::transaction::WasmScript;
use crate::common::{error::WasmError, types::WasmNetwork};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAddress {
    inner: Address,
}

impl<'de> Deserialize<'de> for WasmAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the inner address as String
        let inner_str: String = Deserialize::deserialize(deserializer)?;

        // Convert the String to Address using From<String> implementation
        let inner_address = Address::from_str(&inner_str).unwrap().assume_checked();

        Ok(WasmAddress { inner: inner_address })
    }
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
