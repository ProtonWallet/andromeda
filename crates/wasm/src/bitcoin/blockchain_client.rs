use std::{collections::HashMap, sync::Arc};

use super::{account::WasmAccount, psbt::WasmPsbt};
use crate::{api::WasmProtonWalletApiClient, common::error::ErrorExt};
use andromeda_api::transaction::{BroadcastMessage, ExchangeRateOrTransactionTime, RecommendedFees};
use andromeda_bitcoin::{
    account_trait::AccessWallet,
    blockchain_client::{self, BlockchainClient, MinimumFees},
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = getDefaultStopGap)]
pub fn get_default_stop_gap() -> usize {
    blockchain_client::DEFAULT_STOP_GAP
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmBlockchainClient {
    inner: Arc<BlockchainClient>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Map<string, number>")]
    pub type FeeRateByBlockEstimation;
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct WasmMinimumFees {
    pub MinimumBroadcastFee: f32,
    pub MinimumIncrementalFee: f32,
}

impl From<MinimumFees> for WasmMinimumFees {
    fn from(value: MinimumFees) -> Self {
        WasmMinimumFees {
            MinimumBroadcastFee: value.MinimumBroadcastFee,
            MinimumIncrementalFee: value.MinimumIncrementalFee,
        }
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct WasmRecommendedFees {
    pub FastestFee: u8,
    pub HalfHourFee: u8,
    pub HourFee: u8,
    pub EconomyFee: u8,
    pub MinimumFee: u8,
}

impl From<RecommendedFees> for WasmRecommendedFees {
    fn from(value: RecommendedFees) -> Self {
        WasmRecommendedFees {
            FastestFee: value.FastestFee,
            HalfHourFee: value.HalfHourFee,
            HourFee: value.HourFee,
            EconomyFee: value.EconomyFee,
            MinimumFee: value.MinimumFee,
        }
    }
}

impl From<&WasmBlockchainClient> for Arc<BlockchainClient> {
    fn from(val: &WasmBlockchainClient) -> Self {
        val.inner.clone()
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

impl From<WasmExchangeRateOrTransactionTime> for ExchangeRateOrTransactionTime {
    fn from(val: WasmExchangeRateOrTransactionTime) -> Self {
        match val.key {
            WasmExchangeRateOrTransactionTimeEnum::ExchangeRate => {
                ExchangeRateOrTransactionTime::ExchangeRate(val.value)
            }
            WasmExchangeRateOrTransactionTimeEnum::TransactionTime => {
                ExchangeRateOrTransactionTime::TransactionTime(val.value)
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
pub struct WasmBroadcastMessage {
    pub data_packet: String,
    pub key_packets: HashMap<String, String>,
}

impl From<WasmBroadcastMessage> for BroadcastMessage {
    fn from(val: WasmBroadcastMessage) -> Self {
        BroadcastMessage {
            DataPacket: val.data_packet,
            KeyPackets: val.key_packets,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEmailIntegrationData {
    address_id: Option<String>,
    body: Option<String>,
    message: Option<WasmBroadcastMessage>,
    recipients: Option<HashMap<String, String>>,
    is_anonymous: Option<u8>,
}

#[wasm_bindgen]
impl WasmBlockchainClient {
    /// Generates a Mnemonic with a random entropy based on the given word
    /// count.
    #[wasm_bindgen(constructor)]
    pub fn new(proton_api_client: &WasmProtonWalletApiClient) -> Result<WasmBlockchainClient, JsValue> {
        let inner = BlockchainClient::new(proton_api_client.into());
        Ok(WasmBlockchainClient { inner: Arc::new(inner) })
    }

    #[wasm_bindgen(js_name = getFeesEstimation)]
    pub async fn get_fees_estimation(&mut self) -> Result<FeeRateByBlockEstimation, JsValue> {
        let fees_estimation = self.inner.get_fees_estimation().await.map_err(|e| e.to_js_error())?;

        Ok(serde_wasm_bindgen::to_value(&fees_estimation).unwrap().into())
    }

    #[wasm_bindgen(js_name = getMininumFees)]
    pub async fn get_minimum_fees(&mut self) -> Result<WasmMinimumFees, JsValue> {
        let minimum_fees = self.inner.get_minimum_fees().await.map_err(|e| e.to_js_error())?;

        Ok(WasmMinimumFees::from(minimum_fees))
    }

    #[wasm_bindgen(js_name = getRecommendedFees)]
    pub async fn get_recommended_fees(&self) -> Result<WasmRecommendedFees, JsValue> {
        let recommended_fees = self.inner.get_recommended_fees().await.map_err(|e| e.to_js_error())?;

        Ok(WasmRecommendedFees::from(recommended_fees))
    }

    #[wasm_bindgen(js_name = fullSync)]
    pub async fn full_sync(&self, account: &WasmAccount, stop_gap: Option<usize>) -> Result<(), JsValue> {
        let account_inner = account.get_inner();
        let update = self
            .inner
            .full_sync(account_inner.as_ref(), stop_gap)
            .await
            .map_err(|e| e.to_js_error())?;

        account_inner.apply_update(update).await.map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = partialSync)]
    pub async fn partial_sync(&self, account: &WasmAccount) -> Result<(), JsValue> {
        let account_inner = account.get_inner();

        let wallet_lock = account_inner.get_wallet().await;
        let update = self
            .inner
            .partial_sync(wallet_lock)
            .await
            .map_err(|e| e.to_js_error())?;

        account_inner.apply_update(update).await.map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = shouldSync)]
    pub async fn should_sync(&self, account: &WasmAccount) -> Result<bool, JsValue> {
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
    ) -> Result<String, JsValue> {
        let tx = psbt.get_inner().extract_tx().map_err(|e| e.to_js_error())?;

        let email_integration_data = email_integration.unwrap_or_default();

        self.inner
            .broadcast(
                tx.clone(),
                wallet_id,
                wallet_account_id,
                transaction_data.label,
                transaction_data.exchange_rate_or_transaction_time.into(),
                email_integration_data.address_id,
                email_integration_data.body,
                email_integration_data.message.map(|m| m.into()),
                email_integration_data.recipients,
                email_integration_data.is_anonymous,
            )
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(tx.compute_txid().to_string())
    }
}
