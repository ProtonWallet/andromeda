use andromeda_bitcoin::LocalUtxo;
use serde::Serialize;

use super::{
    defined::WasmKeychainKind,
    transaction::{WasmOutPoint, WasmScript},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize)]
pub struct WasmUtxo {
    pub value: u64,
    pub outpoint: WasmOutPoint,
    pub script_pubkey: WasmScript,
    pub keychain: WasmKeychainKind,
    pub is_spent: bool,
}

impl Into<WasmUtxo> for LocalUtxo {
    fn into(self) -> WasmUtxo {
        WasmUtxo {
            value: self.txout.value,
            outpoint: self.outpoint.into(),
            script_pubkey: self.txout.script_pubkey.into(),
            keychain: self.keychain.into(),
            is_spent: self.is_spent,
        }
    }
}
