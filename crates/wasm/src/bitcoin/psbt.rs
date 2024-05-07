use andromeda_bitcoin::{error::Error as BitcoinError, Address, PartiallySignedTransaction, SignOptions};
use andromeda_common::Network;
use wasm_bindgen::prelude::*;

use super::account::WasmAccount;
use crate::common::{error::ErrorExt, types::WasmNetwork};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmPsbtRecipient(pub String, pub u64);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmPartiallySignedTransaction {
    inner: PartiallySignedTransaction,

    pub recipients: Vec<WasmPsbtRecipient>,
    pub total_fees: u64,
}

impl WasmPartiallySignedTransaction {
    pub fn get_inner(&self) -> PartiallySignedTransaction {
        self.inner.clone()
    }

    pub fn from_psbt(psbt: &PartiallySignedTransaction, network: Network) -> WasmPartiallySignedTransaction {
        WasmPartiallySignedTransaction {
            inner: psbt.clone(),
            recipients: psbt
                .clone()
                .extract_tx()
                .output
                .into_iter()
                .map(|o| {
                    let addr = Address::from_script(&o.script_pubkey, network.into()).unwrap();
                    WasmPsbtRecipient(addr.to_string(), o.value)
                })
                .collect(),
            total_fees: psbt.fee().unwrap().to_sat(),
        }
    }
}

impl Into<PartiallySignedTransaction> for &WasmPartiallySignedTransaction {
    fn into(self) -> PartiallySignedTransaction {
        self.inner.clone()
    }
}

#[wasm_bindgen]
impl WasmPartiallySignedTransaction {
    pub fn sign(
        &mut self,
        wasm_account: &WasmAccount,
        network: WasmNetwork,
    ) -> Result<WasmPartiallySignedTransaction, js_sys::Error> {
        let inner = wasm_account.get_inner();

        inner
            .read()
            .expect("lock")
            .get_wallet()
            .sign(&mut self.inner, SignOptions::default())
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        Ok(WasmPartiallySignedTransaction::from_psbt(&self.inner, network.into()))
    }
}
