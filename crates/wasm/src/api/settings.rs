use andromeda_api::settings::{BitcoinUnit, FiatCurrency, SettingsClient, UserSettings};
use serde::{Deserialize, Serialize};

use wasm_bindgen::prelude::*;

use crate::common::error::WasmError;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[wasm_bindgen]
pub enum WasmBitcoinUnit {
    BTC,
    MBTC,
    SAT,
}

impl From<BitcoinUnit> for WasmBitcoinUnit {
    fn from(value: BitcoinUnit) -> Self {
        match value {
            BitcoinUnit::BTC => WasmBitcoinUnit::BTC,
            BitcoinUnit::MBTC => WasmBitcoinUnit::MBTC,
            BitcoinUnit::SAT => WasmBitcoinUnit::SAT,
        }
    }
}

impl From<WasmBitcoinUnit> for BitcoinUnit {
    fn from(value: WasmBitcoinUnit) -> Self {
        match value {
            WasmBitcoinUnit::BTC => BitcoinUnit::BTC,
            WasmBitcoinUnit::MBTC => BitcoinUnit::MBTC,
            WasmBitcoinUnit::SAT => BitcoinUnit::SAT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[wasm_bindgen]
pub enum WasmFiatCurrency {
    USD,
    EUR,
    CHF,
}

impl From<FiatCurrency> for WasmFiatCurrency {
    fn from(value: FiatCurrency) -> Self {
        match value {
            FiatCurrency::USD => WasmFiatCurrency::USD,
            FiatCurrency::EUR => WasmFiatCurrency::EUR,
            FiatCurrency::CHF => WasmFiatCurrency::CHF,
        }
    }
}

impl From<WasmFiatCurrency> for FiatCurrency {
    fn from(value: WasmFiatCurrency) -> Self {
        match value {
            WasmFiatCurrency::USD => FiatCurrency::USD,
            WasmFiatCurrency::EUR => FiatCurrency::EUR,
            WasmFiatCurrency::CHF => FiatCurrency::CHF,
        }
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct WasmUserSettings {
    pub BitcoinUnit: WasmBitcoinUnit,
    pub FiatCurrency: WasmFiatCurrency,
    pub HideEmptyUsedAddresses: u8,
    pub ShowWalletRecovery: u8,
    pub TwoFactorAmountThreshold: Option<u64>,
}

impl From<UserSettings> for WasmUserSettings {
    fn from(value: UserSettings) -> Self {
        Self {
            BitcoinUnit: value.BitcoinUnit.into(),
            FiatCurrency: value.FiatCurrency.into(),
            HideEmptyUsedAddresses: value.HideEmptyUsedAddresses,
            ShowWalletRecovery: value.ShowWalletRecovery,
            TwoFactorAmountThreshold: value.TwoFactorAmountThreshold,
        }
    }
}

#[wasm_bindgen]
pub struct WasmSettingsClient(SettingsClient);

impl From<SettingsClient> for WasmSettingsClient {
    fn from(value: SettingsClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmSettingsClient {
    #[wasm_bindgen(js_name="getUserSettings")]
    pub async fn get_user_settings(&self) -> Result<WasmUserSettings, WasmError> {
        self.0
            .get_user_settings()
            .await
            .map_err(|e| e.into())
            .map(|settings| settings.into())
    }

    #[wasm_bindgen(js_name="setBitcoinUnit")]
    pub async fn bitcoin_unit(&self, symbol: WasmBitcoinUnit) -> Result<WasmUserSettings, WasmError> {
        self.0
            .bitcoin_unit(symbol.into())
            .await
            .map_err(|e| e.into())
            .map(|settings| settings.into())
    }

    #[wasm_bindgen(js_name="setFiatCurrency")]
    pub async fn fiat_currency(&self, symbol: WasmFiatCurrency) -> Result<WasmUserSettings, WasmError> {
        self.0
            .fiat_currency(symbol.into())
            .await
            .map_err(|e| e.into())
            .map(|settings| settings.into())
    }

    #[wasm_bindgen(js_name="setTwoFaThreshold")]
    pub async fn two_fa_threshold(&self, amount: u64) -> Result<WasmUserSettings, WasmError> {
        self.0
            .two_fa_threshold(amount)
            .await
            .map_err(|e| e.into())
            .map(|settings| settings.into())
    }

    #[wasm_bindgen(js_name="setHideEmptyUsedAddresses")]
    pub async fn hide_empty_used_addresses(
        &self,
        hide_empty_used_addresses: bool,
    ) -> Result<WasmUserSettings, WasmError> {
        self.0
            .hide_empty_used_addresses(hide_empty_used_addresses)
            .await
            .map_err(|e| e.into())
            .map(|settings| settings.into())
    }
}
