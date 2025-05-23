use std::str::FromStr;

use andromeda_bitcoin::{
    error::Error as BitcoinError,
    psbt::Psbt,
    transactions::{DetailledTxIn, DetailledTxOutput, TransactionDetails, TransactionTime},
    Address, ConsensusParams, OutPoint, ScriptBuf, Sequence, Transaction,
};
use andromeda_macros::WasmJson;
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

impl From<WasmScript> for ScriptBuf {
    fn from(val: WasmScript) -> Self {
        ScriptBuf::from_bytes(val.0)
    }
}

impl From<&WasmScript> for ScriptBuf {
    fn from(val: &WasmScript) -> Self {
        ScriptBuf::from_bytes(val.0.clone())
    }
}

impl From<ScriptBuf> for WasmScript {
    fn from(val: ScriptBuf) -> Self {
        WasmScript(val.to_bytes())
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

impl From<OutPoint> for WasmOutPoint {
    fn from(val: OutPoint) -> Self {
        WasmOutPoint(val.to_string())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WasmSequence(pub u32);

impl From<Sequence> for WasmSequence {
    fn from(val: Sequence) -> Self {
        WasmSequence(val.0)
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

impl From<DetailledTxIn> for WasmDetailledTxIn {
    fn from(val: DetailledTxIn) -> Self {
        WasmDetailledTxIn {
            previous_output: val.previous_output.map(|o| o.into()),
            script_sig: val.script_sig.into(),
            sequence: val.sequence.into(),
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

impl From<DetailledTxOutput> for WasmTxOut {
    fn from(val: DetailledTxOutput) -> Self {
        WasmTxOut {
            value: val.value,
            script_pubkey: val.script_pubkey.into(),
            address: val.address.map(|a| a.to_string()),
            is_mine: val.is_mine,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Serialize, Deserialize, WasmJson)]
pub struct WasmTransactionDetails {
    inner: TransactionDetails,
}

#[wasm_bindgen]
impl WasmTransactionDetails {
    #[wasm_bindgen(getter)]
    pub fn txid(&self) -> String {
        self.inner.txid.to_string()
    }

    #[wasm_bindgen(js_name = isSend)]
    pub fn is_send(&self) -> bool {
        self.inner.is_send()
    }

    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self) -> u64 {
        self.inner.get_value()
    }

    #[wasm_bindgen(js_name = getValueWithFee)]
    pub fn get_value_with_fee(&self) -> u64 {
        self.inner.get_value_with_fee()
    }

    #[wasm_bindgen(js_name = getAccountDerivationPath)]
    pub fn get_account_derivation_path(&self) -> String {
        self.inner.account_derivation_path.to_string()
    }

    #[wasm_bindgen(js_name = getFee)]
    pub fn get_fee(&self) -> u64 {
        self.inner.fees.unwrap_or(0)
    }

    #[wasm_bindgen(js_name = getSize)]
    pub fn get_size(&self) -> f64 {
        self.inner.vbytes_size as f64
    }

    #[wasm_bindgen(js_name = getTime)]
    pub fn get_transaction_time(&self) -> WasmTransactionTime {
        self.inner.time.into()
    }

    #[wasm_bindgen(js_name = getOutputs)]
    pub fn get_outputs(&self) -> Vec<WasmTxOut> {
        self.inner.outputs.iter().map(|output| output.clone().into()).collect()
    }
}

impl From<TransactionDetails> for WasmTransactionDetails {
    fn from(val: TransactionDetails) -> Self {
        WasmTransactionDetails { inner: val }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmTransactionDetailsData {
    pub Data: WasmTransactionDetails,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmTransactionDetailsArray(pub Vec<WasmTransactionDetailsData>);

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

impl From<TransactionTime> for WasmTransactionTime {
    fn from(val: TransactionTime) -> Self {
        match val {
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
