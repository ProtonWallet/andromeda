use proton_wallet_common::LocalUtxo;

use super::{
    defined::WasmKeychainKind,
    transaction::{WasmConfirmation, WasmOutPoint, WasmScript},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct WasmUtxo {
    pub value: u64,
    pub outpoint: WasmOutPoint,
    pub script_pubkey: WasmScript,
    pub keychain: WasmKeychainKind,
    pub is_spent: bool,
    pub derivation_index: u32,
    pub confirmation_time: WasmConfirmation,
}

impl Into<WasmUtxo> for LocalUtxo {
    fn into(self) -> WasmUtxo {
        WasmUtxo {
            value: self.txout.value,
            outpoint: self.outpoint.into(),
            script_pubkey: self.txout.script_pubkey.into(),
            keychain: self.keychain.into(),
            is_spent: self.is_spent,
            derivation_index: self.derivation_index,
            confirmation_time: self.confirmation_time.into(),
        }
    }
}