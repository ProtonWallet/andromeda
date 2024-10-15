use std::str::FromStr;

use andromeda_bitcoin::{
    error::Error as BitcoinError,
    psbt::Psbt,
    transactions::{DetailledTxIn, DetailledTxOutput, TransactionDetails, TransactionTime},
    Address, ConsensusParams, OutPoint, ScriptBuf, Sequence, Transaction,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{
    super::{account::WasmAccount, psbt::WasmPsbt},
    address::WasmAddress,
};
use crate::common::{error::ErrorExt, types::WasmNetwork};

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
pub struct WasmTransaction {
    inner: Transaction,
}

impl WasmTransaction {
    pub fn get_inner(&self) -> Transaction {
        self.inner.clone()
    }
}

#[wasm_bindgen]
impl WasmTransaction {
    #[wasm_bindgen(js_name = fromPsbt)]
    pub fn from_psbt(value: WasmPsbt) -> Result<WasmTransaction, js_sys::Error> {
        Ok(WasmTransaction {
            inner: value.get_inner().extract_tx().map_err(|e| e.to_js_error())?,
        })
    }
}

#[wasm_bindgen]
impl WasmScript {
    #[wasm_bindgen(js_name = toAddress)]
    pub fn to_address(&self, network: WasmNetwork) -> Result<WasmAddress, js_sys::Error> {
        let script_buf: ScriptBuf = self.into();
        let address = Address::from_script(script_buf.as_script(), ConsensusParams::new(network.into()))
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        Ok(address.into())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
/// Serialised Outpoint under the form <txid>:<index>
pub struct WasmOutPoint(pub String);

#[wasm_bindgen]
impl WasmOutPoint {
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(str: String) -> WasmOutPoint {
        WasmOutPoint(str)
    }
}

impl TryInto<OutPoint> for WasmOutPoint {
    type Error = js_sys::Error;

    fn try_into(self) -> Result<OutPoint, js_sys::Error> {
        OutPoint::from_str(&self.0).map_err(|_| js_sys::Error::new("Could not parse outpoint"))
    }
}

impl Into<WasmOutPoint> for OutPoint {
    fn into(self) -> WasmOutPoint {
        WasmOutPoint(self.to_string())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmSequence(pub u32);

impl Into<WasmSequence> for Sequence {
    fn into(self) -> WasmSequence {
        WasmSequence(self.0)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmDetailledTxIn {
    pub previous_output: Option<WasmTxOut>,
    pub script_sig: WasmScript,
    pub sequence: WasmSequence,
    // pub witness: Vec<u8>, Skip this for now as not needed and didn't find convenient serialisation way
}

impl Into<WasmDetailledTxIn> for DetailledTxIn {
    fn into(self) -> WasmDetailledTxIn {
        WasmDetailledTxIn {
            previous_output: self.previous_output.map(|o| o.into()),
            script_sig: self.script_sig.into(),
            sequence: self.sequence.into(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Tsify, Clone, Serialize, Deserialize)]
pub struct WasmTxOut {
    pub value: u64,
    pub script_pubkey: WasmScript,
    pub is_mine: bool,
    pub address: Option<String>,
}

impl Into<WasmTxOut> for DetailledTxOutput {
    fn into(self) -> WasmTxOut {
        WasmTxOut {
            value: self.value,
            script_pubkey: self.script_pubkey.into(),
            address: self.address.map(|a| a.to_string()),
            is_mine: self.is_mine,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmTransactionDetails {
    pub txid: String,
    pub received: u64,
    pub sent: u64,
    pub fee: Option<u64>,
    pub size: u64,
    pub time: WasmTransactionTime,
    pub inputs: Vec<WasmDetailledTxIn>,
    pub outputs: Vec<WasmTxOut>,
    pub account_derivation_path: String,
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmTransactionDetailsData {
    pub Data: WasmTransactionDetails,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmTransactionDetailsArray(pub Vec<WasmTransactionDetailsData>);

impl Into<WasmTransactionDetails> for TransactionDetails {
    fn into(self) -> WasmTransactionDetails {
        WasmTransactionDetails {
            txid: self.txid.to_string(),
            received: self.received,
            sent: self.sent,
            fee: self.fees,
            size: self.vbytes_size,
            time: self.time.into(),
            inputs: self.inputs.into_iter().map(|input| input.into()).collect::<Vec<_>>(),
            outputs: self.outputs.into_iter().map(|output| output.into()).collect::<Vec<_>>(),
            account_derivation_path: self.account_derivation_path.to_string(),
        }
    }
}

#[wasm_bindgen(js_name = createTransactionFromPsbt)]
pub async fn create_transaction_from_psbt(
    psbt: &WasmPsbt,
    account: &WasmAccount,
) -> Result<WasmTransactionDetailsData, js_sys::Error> {
    let psbt: Psbt = psbt.into();

    let tx = TransactionDetails::from_psbt(&psbt, account.get_inner())
        .await
        .map_err(|e| e.to_js_error())?;

    Ok(WasmTransactionDetailsData { Data: tx.into() })
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmTransactionTime {
    pub confirmed: bool,
    pub confirmation_time: Option<u64>,
    pub last_seen: Option<u64>,
}

impl Into<WasmTransactionTime> for TransactionTime {
    fn into(self) -> WasmTransactionTime {
        match self {
            TransactionTime::Confirmed { confirmation_time } => WasmTransactionTime {
                confirmed: true,
                confirmation_time: Some(confirmation_time),
                last_seen: None,
            },
            TransactionTime::Unconfirmed { last_seen } => WasmTransactionTime {
                confirmed: false,
                confirmation_time: None,
                last_seen: Some(last_seen),
            },
        }
    }
}
