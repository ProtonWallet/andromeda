use super::account::WasmAccount;
use crate::common::{error::ErrorExt, types::WasmNetwork};
use andromeda_bitcoin::{
    account_trait::AccessWallet, error::Error as BitcoinError, psbt::Psbt, Address, ConsensusParams, SignOptions,
};
use andromeda_common::Network;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmPsbtRecipient(pub String, pub u64);

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmPsbt {
    inner: Psbt,

    pub recipients: Vec<WasmPsbtRecipient>,
    pub total_fees: u64,
    pub outputs_amount: u64,
    pub public_address: Option<String>,
}

impl WasmPsbt {
    pub fn get_inner(&self) -> Psbt {
        self.inner.clone()
    }

    pub fn from_psbt(psbt: &Psbt, network: Network) -> Result<WasmPsbt, JsValue> {
        let psbt = WasmPsbt {
            inner: psbt.clone(),
            recipients: psbt
                .clone()
                .extract_tx()
                .map_err(|e| e.to_js_error())?
                .output
                .into_iter()
                .map(|o| {
                    let addr = Address::from_script(&o.script_pubkey, ConsensusParams::new(network.into())).unwrap();
                    WasmPsbtRecipient(addr.to_string(), o.value.to_sat())
                })
                .collect(),
            total_fees: psbt.fee().unwrap().to_sat(),
            outputs_amount: psbt.outputs_amount().unwrap().to_sat(),
            public_address: None,
        };

        Ok(psbt)
    }

    pub fn from_paper_account_psbt(psbt: &Psbt, network: Network, public_address: String) -> Result<WasmPsbt, JsValue> {
        let psbt = WasmPsbt {
            inner: psbt.clone(),
            recipients: psbt
                .clone()
                .extract_tx()
                .map_err(|e| e.to_js_error())?
                .output
                .into_iter()
                .map(|o| {
                    let addr = Address::from_script(&o.script_pubkey, ConsensusParams::new(network.into())).unwrap();
                    WasmPsbtRecipient(addr.to_string(), o.value.to_sat())
                })
                .collect(),
            total_fees: psbt.fee().unwrap().to_sat(),
            outputs_amount: psbt.outputs_amount().unwrap().to_sat(),
            public_address: public_address.into(),
        };

        Ok(psbt)
    }
}

impl From<&WasmPsbt> for Psbt {
    fn from(val: &WasmPsbt) -> Self {
        val.inner.clone()
    }
}

#[wasm_bindgen]
impl WasmPsbt {
    pub async fn sign(&mut self, wasm_account: &WasmAccount, network: WasmNetwork) -> Result<WasmPsbt, JsValue> {
        let inner = wasm_account.get_inner();

        let mut mutable_psbt = self.inner.inner().clone();

        inner
            .lock_wallet()
            .await
            .sign(&mut mutable_psbt, SignOptions::default())
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        WasmPsbt::from_psbt(&mutable_psbt.into(), network.into())
    }

    #[wasm_bindgen(js_name = computeTxVbytes)]
    pub fn compute_tx_vbytes(&self) -> Result<u64, JsValue> {
        self.inner.compute_tx_vbytes().map_err(|e| e.to_js_error())
    }
}
