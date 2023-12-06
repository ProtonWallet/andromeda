use std::str::FromStr;

use crate::error::WasmError;

use super::locktime::WasmLockTime;
use proton_wallet_common::{
    account::SimpleTransaction, bitcoin::Script, ChainPosition, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmScript(pub Vec<u8>);

impl Into<Script> for WasmScript {
    fn into(self) -> Script {
        Script::new(self.0)
    }
}

impl Into<WasmScript> for Script {
    fn into(self) -> WasmScript {
        WasmScript(self.to_bytes())
    }
}

impl Into<WasmScript> for ScriptBuf {
    fn into(self) -> WasmScript {
        WasmScript(self.to_bytes())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
/// Serialised Outpoint under the form <txid>:<index>
pub struct WasmOutPoint(String);

impl TryInto<OutPoint> for WasmOutPoint {
    type Error = WasmError;

    fn try_into(self) -> Result<OutPoint, WasmError> {
        OutPoint::from_str(&self.0).map_err(|_| WasmError::OutpointParsingError)
    }
}

impl Into<WasmOutPoint> for OutPoint {
    fn into(self) -> WasmOutPoint {
        WasmOutPoint(self.to_string())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmSequence(pub u32);

impl Into<WasmSequence> for Sequence {
    fn into(self) -> WasmSequence {
        WasmSequence(self.0)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmTxIn {
    pub previous_output: WasmOutPoint,
    pub script_sig: WasmScript,
    pub sequence: WasmSequence,
    // pub witness: WasmWitness, SKIPPY for now, not needed
}

impl Into<WasmTxIn> for TxIn {
    fn into(self) -> WasmTxIn {
        WasmTxIn {
            previous_output: self.previous_output.into(),
            script_sig: self.script_sig.into(),
            sequence: self.sequence.into(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmTxOut {
    pub value: u64,
    pub script_pubkey: WasmScript,
}

impl Into<WasmTxOut> for TxOut {
    fn into(self) -> WasmTxOut {
        WasmTxOut {
            value: self.value,
            script_pubkey: self.script_pubkey.into(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmTransaction {
    pub version: i32,
    pub lock_time: WasmLockTime,
    pub input: Vec<WasmTxIn>,
    pub output: Vec<WasmTxOut>,
}

impl Into<WasmTransaction> for Transaction {
    fn into(self) -> WasmTransaction {
        WasmTransaction {
            version: self.version,
            lock_time: self.lock_time.into(),
            input: self.input.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
            output: self.output.into_iter().map(|o| o.into()).collect::<Vec<_>>(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmConfirmation {
    pub confirmed: bool,
    pub confirmation_time: Option<u64>,
    pub last_seen: Option<u64>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmSimpleTransaction {
    pub txid: String,
    pub value: i64,
    pub fees: Option<u64>,
    pub confirmation: WasmConfirmation,
}

impl Into<WasmSimpleTransaction> for SimpleTransaction<'_> {
    fn into(self) -> WasmSimpleTransaction {
        WasmSimpleTransaction {
            txid: self.txid.to_string(),
            value: self.value,
            fees: self.fees,
            confirmation: match self.confirmation {
                ChainPosition::Confirmed(anchor) => WasmConfirmation {
                    confirmed: true,
                    confirmation_time: Some(anchor.confirmation_time),
                    last_seen: None,
                },
                ChainPosition::Unconfirmed(last_seen) => WasmConfirmation {
                    confirmed: false,
                    confirmation_time: None,
                    last_seen: Some(last_seen),
                },
            },
        }
    }
}
