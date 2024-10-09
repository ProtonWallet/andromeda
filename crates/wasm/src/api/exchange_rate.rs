use andromeda_api::exchange_rate::{ApiExchangeRate, ApiFiatCurrency, ExchangeRateClient};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::settings::WasmFiatCurrencySymbol;
use crate::common::{error::ErrorExt, types::WasmBitcoinUnit};

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiExchangeRate {
    pub ID: String,
    pub BitcoinUnit: WasmBitcoinUnit,
    pub FiatCurrency: WasmFiatCurrencySymbol,
    pub Sign: Option<String>,
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
            Sign: value.Sign,
            ExchangeRateTime: value.ExchangeRateTime,
            ExchangeRate: value.ExchangeRate,
            Cents: value.Cents,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiFiatCurrency {
    pub ID: String,
    pub Name: String,
    pub Symbol: WasmFiatCurrencySymbol,
    pub Sign: String,
    pub Cents: u64,
}

impl From<ApiFiatCurrency> for WasmApiFiatCurrency {
    fn from(value: ApiFiatCurrency) -> Self {
        WasmApiFiatCurrency {
            ID: value.ID,
            Name: value.Name,
            Symbol: value.Symbol.into(),
            Sign: value.Sign,
            Cents: value.Cents,
        }
    }
}

// We need this wrapper because, tsify doesn't support intoJs in async fns
#[wasm_bindgen(getter_with_clone)]
#[allow(non_snake_case)]
pub struct WasmApiExchangeRateData {
    pub Data: WasmApiExchangeRate,
}

// We need this wrapper because, tsify doesn't support intoJs in async fns
#[wasm_bindgen(getter_with_clone)]
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct WasmApiFiatCurrencyData {
    pub Data: WasmApiFiatCurrency,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiFiatCurrencies(pub Vec<WasmApiFiatCurrencyData>);

#[wasm_bindgen]
#[derive(Clone)]
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
        fiat: WasmFiatCurrencySymbol,
        time: Option<u64>,
    ) -> Result<WasmApiExchangeRateData, JsValue> {
        self.0
            .get_exchange_rate(fiat.into(), time)
            .await
            .map(|n| WasmApiExchangeRateData { Data: n.into() })
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getAllFiatCurrencies")]
    pub async fn get_all_fiat_currencies(&self) -> Result<WasmApiFiatCurrencies, JsValue> {
        let currencies = self
            .0
            .get_all_fiat_currencies()
            .await
            .map(|n| {
                n.into_iter()
                    .map(|f| WasmApiFiatCurrencyData { Data: f.into() })
                    .collect::<Vec<_>>()
            })
            .map_err(|e| e.to_js_error())?;

        Ok(WasmApiFiatCurrencies(currencies))
    }
}
