use andromeda_api::exchange_rate::{ApiExchangeRate, ExchangeRateClient};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::settings::WasmFiatCurrency;
use crate::common::{error::ErrorExt, types::WasmBitcoinUnit};

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiExchangeRate {
    pub ID: String,
    pub BitcoinUnit: WasmBitcoinUnit,
    pub FiatCurrency: WasmFiatCurrency,
    pub ExchangeRateTime: String,
    pub ExchangeRate: u64,
    pub Cents: u64,
}

impl From<ApiExchangeRate> for WasmApiExchangeRate {
    fn from(value: ApiExchangeRate) -> Self {
        Self {
            ID: value.ID,
            BitcoinUnit: value.BitcoinUnit.into(),
            FiatCurrency: value.FiatCurrency.into(),
            ExchangeRateTime: value.ExchangeRateTime,
            ExchangeRate: value.ExchangeRate,
            Cents: value.Cents,
        }
    }
}

// We need this wrapper because, tsify doesn't support intoJs in async fns
#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiExchangeRateData(pub WasmApiExchangeRate);

#[wasm_bindgen]
pub struct WasmExchangeRateClient(ExchangeRateClient);

impl From<ExchangeRateClient> for WasmExchangeRateClient {
    fn from(value: ExchangeRateClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmExchangeRateClient {
    #[wasm_bindgen(js_name = "getExchangeRate")]
    pub async fn get_exchange_rate(
        &self,
        fiat: WasmFiatCurrency,
        time: u64,
    ) -> Result<WasmApiExchangeRateData, js_sys::Error> {
        self.0
            .get_exchange_rate(fiat.into(), Some(time))
            .await
            .map(|n| WasmApiExchangeRateData(n.into()))
            .map_err(|e| e.to_js_error())
    }
}
