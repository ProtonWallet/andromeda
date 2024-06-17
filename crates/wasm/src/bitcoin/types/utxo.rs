use andromeda_bitcoin::LocalOutput;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::transaction::{WasmOutPoint, WasmScript};
use crate::common::types::WasmKeychainKind;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize)]
pub struct WasmUtxo {
    pub value: u64,
    pub outpoint: WasmOutPoint,
    pub script_pubkey: WasmScript,
    pub keychain: WasmKeychainKind,
    pub is_spent: bool,
}

impl Into<WasmUtxo> for LocalOutput {
    fn into(self) -> WasmUtxo {
        WasmUtxo {
            value: self.txout.value.to_sat(),
            outpoint: self.outpoint.into(),
            script_pubkey: self.txout.script_pubkey.into(),
            keychain: self.keychain.into(),
            is_spent: self.is_spent,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmUtxoArray(pub Vec<WasmUtxo>);
