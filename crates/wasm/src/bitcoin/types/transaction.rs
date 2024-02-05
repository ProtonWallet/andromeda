use std::str::FromStr;

use super::super::{account::WasmAccount, psbt::WasmPartiallySignedTransaction};

use crate::common::error::{DetailledWasmError, WasmError};

use andromeda_bitcoin::{
    transactions::{DetailledTxOutput, SimpleTransaction, TransactionDetails},
    Address, OutPoint, PartiallySignedTransaction, ScriptBuf, Sequence, TxIn,
};
use wasm_bindgen::prelude::*;

use super::{
    address::WasmAddress, defined::WasmNetwork, derivation_path::WasmDerivationPath,
    typescript_interfaces::IWasmOutpoint,
};
use andromeda_bitcoin::BdkBlockTime;
use serde::{Deserialize, Serialize};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmScript(pub Vec<u8>);

impl Into<ScriptBuf> for WasmScript {
    fn into(self) -> ScriptBuf {
        ScriptBuf::from_bytes(self.0)
    }
}

impl Into<ScriptBuf> for &WasmScript {
    fn into(self) -> ScriptBuf {
        ScriptBuf::from_bytes(self.0.clone())
    }
}

impl Into<WasmScript> for ScriptBuf {
    fn into(self) -> WasmScript {
        WasmScript(self.to_bytes())
    }
}

#[wasm_bindgen]
impl WasmScript {
    #[wasm_bindgen(js_name = toAddress)]
    pub fn to_address(&self, network: WasmNetwork) -> Result<WasmAddress, DetailledWasmError> {
        let script_bug: ScriptBuf = self.into();
        let address = Address::from_script(script_bug.as_script(), network.into())
            .map_err(|_| WasmError::CannotGetAddressFromScript.into())?;

        Ok(address.into())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
/// Serialised Outpoint under the form <txid>:<index>
pub struct WasmOutPoint(pub String);

#[wasm_bindgen]
impl WasmOutPoint {
    #[wasm_bindgen(js_name = fromRawTs)]
    pub fn from_raw_ts(raw_ts: IWasmOutpoint) -> WasmOutPoint {
        serde_wasm_bindgen::from_value(raw_ts.into()).unwrap()
    }
}

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
    pub address: WasmAddress,
    pub is_mine: bool,
}

impl Into<WasmTxOut> for DetailledTxOutput {
    fn into(self) -> WasmTxOut {
        WasmTxOut {
            value: self.value,
            script_pubkey: self.script_pubkey.into(),
            address: self.address.into(),
            is_mine: self.is_mine,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmTransactionDetails {
    pub txid: String,
    pub received: u64,
    pub sent: u64,
    pub fee: Option<u64>,
    pub confirmation_time: Option<WasmBlockTime>,
    pub inputs: Vec<WasmTxIn>,
    pub outputs: Vec<WasmTxOut>,
}

impl Into<WasmTransactionDetails> for TransactionDetails {
    fn into(self) -> WasmTransactionDetails {
        WasmTransactionDetails {
            txid: self.txid.to_string(),
            received: self.received,
            sent: self.sent,
            fee: self.fees,
            confirmation_time: self.confirmation_time.map(|t| t.into()),
            inputs: self.inputs.into_iter().map(|input| input.into()).collect::<Vec<_>>(),
            outputs: self.outputs.into_iter().map(|output| output.into()).collect::<Vec<_>>(),
        }
    }
}

#[wasm_bindgen]
impl WasmTransactionDetails {
    #[wasm_bindgen(js_name = fromPsbt)]
    pub async fn from_psbt(
        psbt: &WasmPartiallySignedTransaction,
        account: &WasmAccount,
    ) -> Result<WasmTransactionDetails, DetailledWasmError> {
        let psbt: PartiallySignedTransaction = psbt.into();
        let inner = account.get_inner();
        let account = inner.read().expect("lock");
        let wallet = account.get_wallet();

        let tx = TransactionDetails::from_psbt(&psbt, wallet).map_err(|e| e.into())?;
        Ok(tx.into())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmBlockTime {
    pub height: u32,
    pub timestamp: u64,
}

impl Into<WasmBlockTime> for BdkBlockTime {
    fn into(self) -> WasmBlockTime {
        WasmBlockTime {
            height: self.height,
            timestamp: self.timestamp,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize)]
pub struct WasmSimpleTransaction {
    pub txid: String,
    pub received: u64,
    pub sent: u64,
    pub fees: Option<u64>,
    pub confirmation_time: Option<WasmBlockTime>,
    pub account_key: Option<WasmDerivationPath>,
}

impl Into<WasmSimpleTransaction> for SimpleTransaction {
    fn into(self) -> WasmSimpleTransaction {
        WasmSimpleTransaction {
            txid: self.txid.to_string(),
            received: self.received,
            sent: self.sent,
            fees: self.fees,
            confirmation_time: self.confirmation_time.map(|t| t.into()),
            account_key: self.account_key.map(|k| k.into()),
        }
    }
}
