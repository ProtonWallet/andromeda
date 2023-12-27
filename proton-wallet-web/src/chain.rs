use proton_wallet_common::{chain::Chain, client::Client};
use wasm_bindgen::prelude::*;
use web_sys::console::log_2;

use crate::{
    account::WasmAccount,
    error::{DetailledWasmError, WasmError},
    psbt::WasmPartiallySignedTransaction,
};

#[wasm_bindgen(getter_with_clone)]
pub struct WasmChain {
    inner: Chain,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

impl Into<Chain> for WasmChain {
    fn into(self) -> Chain {
        self.inner
    }
}

#[wasm_bindgen]
impl WasmChain {
    /// Generates a Mnemonic with a random entropy based on the given word count.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmChain, DetailledWasmError> {
        let client = Client::new(None).unwrap(); // TODO: change this
        let inner = Chain::new(client.inner());

        Ok(WasmChain { inner })
    }

    #[wasm_bindgen]
    pub async fn get_fees_estimation(&self) -> FeeRateByBlockEstimation {
        let fees_estimation = self.inner.get_fees_estimation().await.unwrap_or_default();
        serde_wasm_bindgen::to_value(&fees_estimation).unwrap().into()
    }

    pub async fn full_sync(&self, account: &WasmAccount) -> Result<(), DetailledWasmError> {
        log_2(
            &"Start full sync".into(),
            &account.get_derivation_path().unwrap().to_string().into(),
        );

        let account_inner = account.get_inner();

        self.inner
            .full_sync(
                account_inner
                    .write()
                    .await
                    .map_err(|_| WasmError::LockError.into())?
                    .get_mutable_wallet(),
            )
            .await
            .map_err(|e| e.into())?;

        account_inner.release_write_lock();
        log_2(
            &"Finished full sync".into(),
            &account.get_derivation_path().unwrap().to_string().into(),
        );
        Ok(())
    }

    pub async fn partial_sync(&self, account: &WasmAccount) -> Result<(), DetailledWasmError> {
        log_2(
            &"Start part. sync".into(),
            &account.get_derivation_path().unwrap().to_string().into(),
        );

        let account_inner = account.get_inner();

        self.inner
            .partial_sync(
                account_inner
                    .write()
                    .await
                    .map_err(|_| WasmError::LockError.into())?
                    .get_mutable_wallet(),
            )
            .await
            .map_err(|e| e.into())?;

        account_inner.release_write_lock();
        log_2(
            &"Finished part. sync".into(),
            &account.get_derivation_path().unwrap().to_string().into(),
        );
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn broadcast_psbt(&self, psbt: &WasmPartiallySignedTransaction) -> Result<String, WasmError> {
        let tx = psbt.get_inner().extract_tx();

        self.inner
            .broadcast(tx.clone())
            .await
            .map_err(|_| WasmError::ChecksumMismatch)?;

        Ok(tx.txid().to_string())
    }
}
