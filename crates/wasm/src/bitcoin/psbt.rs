use andromeda_bitcoin::{error::Error as BitcoinError, psbt::Psbt, Address, ConsensusParams, SignOptions};
use andromeda_common::Network;
use wasm_bindgen::prelude::*;

use super::account::WasmAccount;
use crate::common::{error::ErrorExt, types::WasmNetwork};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmPsbtRecipient(pub String, pub u64);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmPsbt {
    inner: Psbt,

    pub recipients: Vec<WasmPsbtRecipient>,
    pub total_fees: u64,
}

impl WasmPsbt {
    pub fn get_inner(&self) -> Psbt {
        self.inner.clone()
    }

    pub fn from_psbt(psbt: &Psbt, network: Network) -> Result<WasmPsbt, js_sys::Error> {
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
        };

        Ok(psbt)
    }
}

impl Into<Psbt> for &WasmPsbt {
    fn into(self) -> Psbt {
        self.inner.clone()
    }
}

#[wasm_bindgen]
impl WasmPsbt {
    pub async fn sign(&mut self, wasm_account: &WasmAccount, network: WasmNetwork) -> Result<WasmPsbt, js_sys::Error> {
        let inner = wasm_account.get_inner();

        let mut mutable_psbt = self.inner.inner().clone();

        inner
            .get_wallet()
            .await
            .sign(&mut mutable_psbt, SignOptions::default())
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        WasmPsbt::from_psbt(&mutable_psbt.into(), network.into())
    }
}
