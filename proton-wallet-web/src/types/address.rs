use std::str::FromStr;

use proton_wallet_common::{bitcoin::Network, Address, ScriptBuf};
use wasm_bindgen::prelude::*;

use crate::error::WasmError;

use super::{defined::WasmNetwork, transaction::WasmScript};

#[wasm_bindgen]
pub struct WasmAddress {
    inner: Address,
}

impl Into<Address> for &WasmAddress {
    fn into(self) -> Address {
        let addr = self.inner.clone();
        addr
    }
}

#[wasm_bindgen]
impl WasmAddress {
    #[wasm_bindgen(constructor)]
    pub fn new(str: String) -> WasmAddress {
        let inner = Address::from_str(&str)
            .unwrap()
            .require_network(Network::Testnet.into())
            .unwrap();

        println!("inner address: {:?}", inner.to_string());

        WasmAddress { inner }
    }

    #[wasm_bindgen]
    pub fn from_script(value: WasmScript, network: WasmNetwork) -> Result<WasmAddress, WasmError> {
        let network: Network = network.into();

        let script: ScriptBuf = value.into();
        let address = Address::from_script(&script, network.into()).unwrap();

        Ok(WasmAddress { inner: address })
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen]
    pub fn into_script(&self) -> WasmScript {
        self.inner.script_pubkey().into()
    }
}
