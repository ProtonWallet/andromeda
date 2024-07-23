use andromeda_api::price_graph::{PriceGraph, PriceGraphClient, Timeframe};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::settings::WasmFiatCurrencySymbol;
use crate::common::{error::ErrorExt, types::WasmBitcoinUnit};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPriceGraphClient(PriceGraphClient);

impl From<PriceGraphClient> for WasmPriceGraphClient {
    fn from(value: PriceGraphClient) -> Self {
        Self(value)
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmTimeframe {
    OneDay,
    OneWeek,
    OneMonth,
    Unsupported,
}

impl Into<Timeframe> for WasmTimeframe {
    fn into(self: WasmTimeframe) -> Timeframe {
        match self {
            WasmTimeframe::OneDay => Timeframe::OneDay,
            WasmTimeframe::OneWeek => Timeframe::OneWeek,
            WasmTimeframe::OneMonth => Timeframe::OneMonth,
            WasmTimeframe::Unsupported => Timeframe::Unsupported,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmDataPoint {
    pub ExchangeRate: u32,
    pub Timestamp: u64,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmPriceGraph {
    pub FiatCurrencySymbol: WasmFiatCurrencySymbol,
    pub BitcoinUnitSymbol: WasmBitcoinUnit,
    pub GraphData: Vec<WasmDataPoint>,
}

impl From<PriceGraph> for WasmPriceGraph {
    fn from(value: PriceGraph) -> Self {
        WasmPriceGraph {
            FiatCurrencySymbol: value.FiatCurrencySymbol.into(),
            BitcoinUnitSymbol: value.BitcoinUnitSymbol.into(),
            GraphData: value
                .GraphData
                .into_iter()
                .map(|d| WasmDataPoint {
                    ExchangeRate: d.ExchangeRate,
                    Timestamp: d.Timestamp,
                })
                .collect::<Vec<_>>(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWrappedPriceGraph {
    pub data: WasmPriceGraph,
}

#[wasm_bindgen]
impl WasmPriceGraphClient {
    #[wasm_bindgen(js_name = "getGraphData")]
    pub async fn get_graph_data(
        &self,
        fiat_currency: WasmFiatCurrencySymbol,
        timeframe: WasmTimeframe,
    ) -> Result<WasmWrappedPriceGraph, JsValue> {
        self.0
            .get_graph_data(fiat_currency.into(), timeframe.into())
            .await
            .map(|c| WasmWrappedPriceGraph { data: c.into() })
            .map_err(|e| e.to_js_error())
    }
}
