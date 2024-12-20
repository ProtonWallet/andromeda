use andromeda_bitcoin::Balance;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmBalance {
    /// All coinbase outputs not yet matured
    pub immature: u64,
    /// Unconfirmed UTXOs generated by a wallet tx
    pub trusted_pending: u64,
    /// Unconfirmed UTXOs received from an external wallet
    pub untrusted_pending: u64,
    /// Confirmed and immediately spendable balance
    pub confirmed: u64,
}

impl Into<WasmBalance> for Balance {
    fn into(self) -> WasmBalance {
        WasmBalance {
            immature: self.immature.to_sat(),
            trusted_pending: self.trusted_pending.to_sat(),
            untrusted_pending: self.untrusted_pending.to_sat(),
            confirmed: self.confirmed.to_sat(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize)]
pub struct WasmBalanceWrapper {
    pub data: WasmBalance,
}
