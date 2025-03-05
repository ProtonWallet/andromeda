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

impl From<LocalOutput> for WasmUtxo {
    fn from(val: LocalOutput) -> Self {
        WasmUtxo {
            value: val.txout.value.to_sat(),
            outpoint: val.outpoint.into(),
            script_pubkey: val.txout.script_pubkey.into(),
            keychain: val.keychain.into(),
            is_spent: val.is_spent,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmUtxoArray(pub Vec<WasmUtxo>);
