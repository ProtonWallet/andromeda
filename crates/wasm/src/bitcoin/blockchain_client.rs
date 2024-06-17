use andromeda_api::transaction::ExchangeRateOrTransactionTime;
use andromeda_bitcoin::blockchain_client::{self, BlockchainClient};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{account::WasmAccount, psbt::WasmPsbt};
use crate::{api::WasmProtonWalletApiClient, common::error::ErrorExt};

#[wasm_bindgen(js_name = getDefaultStopGap)]
pub fn get_default_stop_gap() -> usize {
    blockchain_client::DEFAULT_STOP_GAP
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmBlockchainClient {
    inner: BlockchainClient,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

impl Into<BlockchainClient> for WasmBlockchainClient {
    fn into(self) -> BlockchainClient {
        self.inner
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmExchangeRateOrTransactionTimeEnum {
    ExchangeRate,
    TransactionTime,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmExchangeRateOrTransactionTime {
    key: WasmExchangeRateOrTransactionTimeEnum,
    value: String,
}

impl Into<ExchangeRateOrTransactionTime> for WasmExchangeRateOrTransactionTime {
    fn into(self) -> ExchangeRateOrTransactionTime {
        match self.key {
            WasmExchangeRateOrTransactionTimeEnum::ExchangeRate => {
                ExchangeRateOrTransactionTime::ExchangeRate(self.value)
            }
            WasmExchangeRateOrTransactionTimeEnum::TransactionTime => {
                ExchangeRateOrTransactionTime::TransactionTime(self.value)
            }
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmTransactionData {
    label: Option<String>,
    exchange_rate_or_transaction_time: WasmExchangeRateOrTransactionTime,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEmailIntegrationData {
    address_id: Option<String>,
    subject: Option<String>,
    body: Option<String>,
}

#[wasm_bindgen]
impl WasmBlockchainClient {
    /// Generates a Mnemonic with a random entropy based on the given word
    /// count.
    #[wasm_bindgen(constructor)]
    pub fn new(proton_api_client: &WasmProtonWalletApiClient) -> Result<WasmBlockchainClient, js_sys::Error> {
        let inner = BlockchainClient::new(proton_api_client.into());
        Ok(WasmBlockchainClient { inner })
    }

    #[wasm_bindgen(js_name = getFeesEstimation)]
    pub async fn get_fees_estimation(&mut self) -> FeeRateByBlockEstimation {
        let fees_estimation = self.inner.get_fees_estimation().await.unwrap_or_default();
        serde_wasm_bindgen::to_value(&fees_estimation).unwrap().into()
    }

    #[wasm_bindgen(js_name = fullSync)]
    pub async fn full_sync(&self, account: &WasmAccount, stop_gap: Option<usize>) -> Result<(), js_sys::Error> {
        let account_inner = account.get_inner();

        let read_lock = account_inner.get_wallet().await;
        let update = self
            .inner
            .full_sync(read_lock, stop_gap)
            .await
            .map_err(|e| e.to_js_error())?;

        account_inner.store_update(update).await.map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = partialSync)]
    pub async fn partial_sync(&self, account: &WasmAccount) -> Result<(), js_sys::Error> {
        let account_inner = account.get_inner();

        let read_lock = account_inner.get_wallet().await;
        let update = self.inner.partial_sync(read_lock).await.map_err(|e| e.to_js_error())?;

        account_inner.store_update(update).await.map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = shouldSync)]
    pub async fn should_sync(&self, account: &WasmAccount) -> Result<bool, js_sys::Error> {
        let account_inner = account.get_inner();

        let wallet_lock = account_inner.get_wallet().await;

        self.inner.should_sync(wallet_lock).await.map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = broadcastPsbt)]
    pub async fn broadcast_psbt(
        &self,
        psbt: &WasmPsbt,
        wallet_id: String,
        wallet_account_id: String,
        transaction_data: WasmTransactionData,
        email_integration: Option<WasmEmailIntegrationData>,
    ) -> Result<String, js_sys::Error> {
        let tx = psbt.get_inner().extract_tx().map_err(|e| e.to_js_error())?;

        self.inner
            .broadcast(
                tx.clone(),
                wallet_id,
                wallet_account_id,
                transaction_data.label,
                transaction_data.exchange_rate_or_transaction_time.into(),
                email_integration.clone().map(|e| e.address_id).flatten(),
                email_integration.clone().map(|e| e.subject).flatten(),
                email_integration.clone().map(|e| e.body).flatten(),
            )
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(tx.compute_txid().to_string())
    }
}
