use andromeda_bitcoin::{bitcoin::Network, Address, PartiallySignedTransaction, SignOptions};
use wasm_bindgen::prelude::*;

use super::{account::WasmAccount, types::defined::WasmNetwork};
use crate::common::error::{DetailledWasmError, WasmError};

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
    pub async fn sign(
        &mut self,
        wasm_account: &WasmAccount,
        network: WasmNetwork,
    ) -> Result<WasmPartiallySignedTransaction, DetailledWasmError> {
        let inner = wasm_account.get_inner();

        inner
            .write()
            .map_err(|_| WasmError::LockError.into())?
            .get_mutable_wallet()
            .sign(&mut self.inner, SignOptions::default())
            .map_err(|_| WasmError::CannotSignPsbt.into())?;

        Ok(WasmPartiallySignedTransaction::from_psbt(&self.inner, network.into()))
    }
}
