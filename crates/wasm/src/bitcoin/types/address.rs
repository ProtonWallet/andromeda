use std::str::FromStr;

use andromeda_bitcoin::{address::AddressDetails, error::Error as BitcoinError, Address, ConsensusParams, ScriptBuf};
use serde::{Deserialize, Deserializer, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{
    balance::WasmBalance,
    transaction::{WasmScript, WasmTransactionDetails},
};
use crate::common::{
    error::ErrorExt,
    types::{WasmKeychainKind, WasmNetwork},
};

#[wasm_bindgen]
#[derive(Clone, Serialize)]
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
    pub fn new(str: String, network: WasmNetwork) -> Result<WasmAddress, js_sys::Error> {
        let inner = Address::from_str(&str)
            .map_err(|e| BitcoinError::from(e).to_js_error())?
            .require_network(network.into())
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        Ok(WasmAddress { inner })
    }

    #[wasm_bindgen(js_name = fromScript)]
    pub fn from_script(value: WasmScript, network: WasmNetwork) -> Result<WasmAddress, js_sys::Error> {
        let script: ScriptBuf = value.into();

        Ok(WasmAddress {
            inner: Address::from_script(&script, ConsensusParams::new(network.into()))
                .map_err(|e| BitcoinError::from(e).to_js_error())?,
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

// Address Details

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmAddressDetails {
    pub index: u32,
    pub address: String,
    pub transactions: Vec<WasmTransactionDetails>,
    pub balance: WasmBalance,
    pub keychain: WasmKeychainKind,
}

impl Into<WasmAddressDetails> for AddressDetails {
    fn into(self) -> WasmAddressDetails {
        WasmAddressDetails {
            index: self.index,
            address: self.address,
            transactions: self.transactions.into_iter().map(|t| t.into()).collect::<Vec<_>>(),
            balance: self.balance.into(),
            keychain: self.keychain.into(),
        }
    }
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmAddressDetailsData {
    pub Data: WasmAddressDetails,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmAddressDetailsArray(pub Vec<WasmAddressDetailsData>);
