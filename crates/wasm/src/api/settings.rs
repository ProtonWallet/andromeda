use andromeda_api::settings::{FiatCurrency, SettingsClient, UserSettings};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::{error::WasmError, types::WasmBitcoinUnit};

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmFiatCurrency {
    ALL,
    DZD,
    ARS,
    AMD,
    AUD,
    AZN,
    BHD,
    BDT,
    BYN,
    BMD,
    BOB,
    BAM,
    BRL,
    BGN,
    KHR,
    CAD,
    CLP,
    CNY,
    COP,
    CRC,
    HRK,
    CUP,
    CZK,
    DKK,
    DOP,
    EGP,
    EUR,
    GEL,
    GHS,
    GTQ,
    HNL,
    HKD,
    HUF,
    ISK,
    INR,
    IDR,
    IRR,
    IQD,
    ILS,
    JMD,
    JPY,
    JOD,
    KZT,
    KES,
    KWD,
    KGS,
    LBP,
    MKD,
    MYR,
    MUR,
    MXN,
    MDL,
    MNT,
    MAD,
    MMK,
    NAD,
    NPR,
    TWD,
    NZD,
    NIO,
    NGN,
    NOK,
    OMR,
    PKR,
    PAB,
    PEN,
    PHP,
    PLN,
    GBP,
    QAR,
    RON,
    RUB,
    SAR,
    RSD,
    SGD,
    ZAR,
    KRW,
    SSP,
    VES,
    LKR,
    SEK,
    CHF,
    THB,
    TTD,
    TND,
    TRY,
    UGX,
    UAH,
    AED,
    USD,
    UYU,
    UZS,
    VND,
}

impl From<FiatCurrency> for WasmFiatCurrency {
    fn from(value: FiatCurrency) -> Self {
        match value {
            FiatCurrency::ALL => WasmFiatCurrency::ALL,
            FiatCurrency::DZD => WasmFiatCurrency::DZD,
            FiatCurrency::ARS => WasmFiatCurrency::ARS,
            FiatCurrency::AMD => WasmFiatCurrency::AMD,
            FiatCurrency::AUD => WasmFiatCurrency::AUD,
            FiatCurrency::AZN => WasmFiatCurrency::AZN,
            FiatCurrency::BHD => WasmFiatCurrency::BHD,
            FiatCurrency::BDT => WasmFiatCurrency::BDT,
            FiatCurrency::BYN => WasmFiatCurrency::BYN,
            FiatCurrency::BMD => WasmFiatCurrency::BMD,
            FiatCurrency::BOB => WasmFiatCurrency::BOB,
            FiatCurrency::BAM => WasmFiatCurrency::BAM,
            FiatCurrency::BRL => WasmFiatCurrency::BRL,
            FiatCurrency::BGN => WasmFiatCurrency::BGN,
            FiatCurrency::KHR => WasmFiatCurrency::KHR,
            FiatCurrency::CAD => WasmFiatCurrency::CAD,
            FiatCurrency::CLP => WasmFiatCurrency::CLP,
            FiatCurrency::CNY => WasmFiatCurrency::CNY,
            FiatCurrency::COP => WasmFiatCurrency::COP,
            FiatCurrency::CRC => WasmFiatCurrency::CRC,
            FiatCurrency::HRK => WasmFiatCurrency::HRK,
            FiatCurrency::CUP => WasmFiatCurrency::CUP,
            FiatCurrency::CZK => WasmFiatCurrency::CZK,
            FiatCurrency::DKK => WasmFiatCurrency::DKK,
            FiatCurrency::DOP => WasmFiatCurrency::DOP,
            FiatCurrency::EGP => WasmFiatCurrency::EGP,
            FiatCurrency::EUR => WasmFiatCurrency::EUR,
            FiatCurrency::GEL => WasmFiatCurrency::GEL,
            FiatCurrency::GHS => WasmFiatCurrency::GHS,
            FiatCurrency::GTQ => WasmFiatCurrency::GTQ,
            FiatCurrency::HNL => WasmFiatCurrency::HNL,
            FiatCurrency::HKD => WasmFiatCurrency::HKD,
            FiatCurrency::HUF => WasmFiatCurrency::HUF,
            FiatCurrency::ISK => WasmFiatCurrency::ISK,
            FiatCurrency::INR => WasmFiatCurrency::INR,
            FiatCurrency::IDR => WasmFiatCurrency::IDR,
            FiatCurrency::IRR => WasmFiatCurrency::IRR,
            FiatCurrency::IQD => WasmFiatCurrency::IQD,
            FiatCurrency::ILS => WasmFiatCurrency::ILS,
            FiatCurrency::JMD => WasmFiatCurrency::JMD,
            FiatCurrency::JPY => WasmFiatCurrency::JPY,
            FiatCurrency::JOD => WasmFiatCurrency::JOD,
            FiatCurrency::KZT => WasmFiatCurrency::KZT,
            FiatCurrency::KES => WasmFiatCurrency::KES,
            FiatCurrency::KWD => WasmFiatCurrency::KWD,
            FiatCurrency::KGS => WasmFiatCurrency::KGS,
            FiatCurrency::LBP => WasmFiatCurrency::LBP,
            FiatCurrency::MKD => WasmFiatCurrency::MKD,
            FiatCurrency::MYR => WasmFiatCurrency::MYR,
            FiatCurrency::MUR => WasmFiatCurrency::MUR,
            FiatCurrency::MXN => WasmFiatCurrency::MXN,
            FiatCurrency::MDL => WasmFiatCurrency::MDL,
            FiatCurrency::MNT => WasmFiatCurrency::MNT,
            FiatCurrency::MAD => WasmFiatCurrency::MAD,
            FiatCurrency::MMK => WasmFiatCurrency::MMK,
            FiatCurrency::NAD => WasmFiatCurrency::NAD,
            FiatCurrency::NPR => WasmFiatCurrency::NPR,
            FiatCurrency::TWD => WasmFiatCurrency::TWD,
            FiatCurrency::NZD => WasmFiatCurrency::NZD,
            FiatCurrency::NIO => WasmFiatCurrency::NIO,
            FiatCurrency::NGN => WasmFiatCurrency::NGN,
            FiatCurrency::NOK => WasmFiatCurrency::NOK,
            FiatCurrency::OMR => WasmFiatCurrency::OMR,
            FiatCurrency::PKR => WasmFiatCurrency::PKR,
            FiatCurrency::PAB => WasmFiatCurrency::PAB,
            FiatCurrency::PEN => WasmFiatCurrency::PEN,
            FiatCurrency::PHP => WasmFiatCurrency::PHP,
            FiatCurrency::PLN => WasmFiatCurrency::PLN,
            FiatCurrency::GBP => WasmFiatCurrency::GBP,
            FiatCurrency::QAR => WasmFiatCurrency::QAR,
            FiatCurrency::RON => WasmFiatCurrency::RON,
            FiatCurrency::RUB => WasmFiatCurrency::RUB,
            FiatCurrency::SAR => WasmFiatCurrency::SAR,
            FiatCurrency::RSD => WasmFiatCurrency::RSD,
            FiatCurrency::SGD => WasmFiatCurrency::SGD,
            FiatCurrency::ZAR => WasmFiatCurrency::ZAR,
            FiatCurrency::KRW => WasmFiatCurrency::KRW,
            FiatCurrency::SSP => WasmFiatCurrency::SSP,
            FiatCurrency::VES => WasmFiatCurrency::VES,
            FiatCurrency::LKR => WasmFiatCurrency::LKR,
            FiatCurrency::SEK => WasmFiatCurrency::SEK,
            FiatCurrency::CHF => WasmFiatCurrency::CHF,
            FiatCurrency::THB => WasmFiatCurrency::THB,
            FiatCurrency::TTD => WasmFiatCurrency::TTD,
            FiatCurrency::TND => WasmFiatCurrency::TND,
            FiatCurrency::TRY => WasmFiatCurrency::TRY,
            FiatCurrency::UGX => WasmFiatCurrency::UGX,
            FiatCurrency::UAH => WasmFiatCurrency::UAH,
            FiatCurrency::AED => WasmFiatCurrency::AED,
            FiatCurrency::USD => WasmFiatCurrency::USD,
            FiatCurrency::UYU => WasmFiatCurrency::UYU,
            FiatCurrency::UZS => WasmFiatCurrency::UZS,
            FiatCurrency::VND => WasmFiatCurrency::VND,
        }
    }
}

impl From<WasmFiatCurrency> for FiatCurrency {
    fn from(value: WasmFiatCurrency) -> Self {
        match value {
            WasmFiatCurrency::ALL => FiatCurrency::ALL,
            WasmFiatCurrency::DZD => FiatCurrency::DZD,
            WasmFiatCurrency::ARS => FiatCurrency::ARS,
            WasmFiatCurrency::AMD => FiatCurrency::AMD,
            WasmFiatCurrency::AUD => FiatCurrency::AUD,
            WasmFiatCurrency::AZN => FiatCurrency::AZN,
            WasmFiatCurrency::BHD => FiatCurrency::BHD,
            WasmFiatCurrency::BDT => FiatCurrency::BDT,
            WasmFiatCurrency::BYN => FiatCurrency::BYN,
            WasmFiatCurrency::BMD => FiatCurrency::BMD,
            WasmFiatCurrency::BOB => FiatCurrency::BOB,
            WasmFiatCurrency::BAM => FiatCurrency::BAM,
            WasmFiatCurrency::BRL => FiatCurrency::BRL,
            WasmFiatCurrency::BGN => FiatCurrency::BGN,
            WasmFiatCurrency::KHR => FiatCurrency::KHR,
            WasmFiatCurrency::CAD => FiatCurrency::CAD,
            WasmFiatCurrency::CLP => FiatCurrency::CLP,
            WasmFiatCurrency::CNY => FiatCurrency::CNY,
            WasmFiatCurrency::COP => FiatCurrency::COP,
            WasmFiatCurrency::CRC => FiatCurrency::CRC,
            WasmFiatCurrency::HRK => FiatCurrency::HRK,
            WasmFiatCurrency::CUP => FiatCurrency::CUP,
            WasmFiatCurrency::CZK => FiatCurrency::CZK,
            WasmFiatCurrency::DKK => FiatCurrency::DKK,
            WasmFiatCurrency::DOP => FiatCurrency::DOP,
            WasmFiatCurrency::EGP => FiatCurrency::EGP,
            WasmFiatCurrency::EUR => FiatCurrency::EUR,
            WasmFiatCurrency::GEL => FiatCurrency::GEL,
            WasmFiatCurrency::GHS => FiatCurrency::GHS,
            WasmFiatCurrency::GTQ => FiatCurrency::GTQ,
            WasmFiatCurrency::HNL => FiatCurrency::HNL,
            WasmFiatCurrency::HKD => FiatCurrency::HKD,
            WasmFiatCurrency::HUF => FiatCurrency::HUF,
            WasmFiatCurrency::ISK => FiatCurrency::ISK,
            WasmFiatCurrency::INR => FiatCurrency::INR,
            WasmFiatCurrency::IDR => FiatCurrency::IDR,
            WasmFiatCurrency::IRR => FiatCurrency::IRR,
            WasmFiatCurrency::IQD => FiatCurrency::IQD,
            WasmFiatCurrency::ILS => FiatCurrency::ILS,
            WasmFiatCurrency::JMD => FiatCurrency::JMD,
            WasmFiatCurrency::JPY => FiatCurrency::JPY,
            WasmFiatCurrency::JOD => FiatCurrency::JOD,
            WasmFiatCurrency::KZT => FiatCurrency::KZT,
            WasmFiatCurrency::KES => FiatCurrency::KES,
            WasmFiatCurrency::KWD => FiatCurrency::KWD,
            WasmFiatCurrency::KGS => FiatCurrency::KGS,
            WasmFiatCurrency::LBP => FiatCurrency::LBP,
            WasmFiatCurrency::MKD => FiatCurrency::MKD,
            WasmFiatCurrency::MYR => FiatCurrency::MYR,
            WasmFiatCurrency::MUR => FiatCurrency::MUR,
            WasmFiatCurrency::MXN => FiatCurrency::MXN,
            WasmFiatCurrency::MDL => FiatCurrency::MDL,
            WasmFiatCurrency::MNT => FiatCurrency::MNT,
            WasmFiatCurrency::MAD => FiatCurrency::MAD,
            WasmFiatCurrency::MMK => FiatCurrency::MMK,
            WasmFiatCurrency::NAD => FiatCurrency::NAD,
            WasmFiatCurrency::NPR => FiatCurrency::NPR,
            WasmFiatCurrency::TWD => FiatCurrency::TWD,
            WasmFiatCurrency::NZD => FiatCurrency::NZD,
            WasmFiatCurrency::NIO => FiatCurrency::NIO,
            WasmFiatCurrency::NGN => FiatCurrency::NGN,
            WasmFiatCurrency::NOK => FiatCurrency::NOK,
            WasmFiatCurrency::OMR => FiatCurrency::OMR,
            WasmFiatCurrency::PKR => FiatCurrency::PKR,
            WasmFiatCurrency::PAB => FiatCurrency::PAB,
            WasmFiatCurrency::PEN => FiatCurrency::PEN,
            WasmFiatCurrency::PHP => FiatCurrency::PHP,
            WasmFiatCurrency::PLN => FiatCurrency::PLN,
            WasmFiatCurrency::GBP => FiatCurrency::GBP,
            WasmFiatCurrency::QAR => FiatCurrency::QAR,
            WasmFiatCurrency::RON => FiatCurrency::RON,
            WasmFiatCurrency::RUB => FiatCurrency::RUB,
            WasmFiatCurrency::SAR => FiatCurrency::SAR,
            WasmFiatCurrency::RSD => FiatCurrency::RSD,
            WasmFiatCurrency::SGD => FiatCurrency::SGD,
            WasmFiatCurrency::ZAR => FiatCurrency::ZAR,
            WasmFiatCurrency::KRW => FiatCurrency::KRW,
            WasmFiatCurrency::SSP => FiatCurrency::SSP,
            WasmFiatCurrency::VES => FiatCurrency::VES,
            WasmFiatCurrency::LKR => FiatCurrency::LKR,
            WasmFiatCurrency::SEK => FiatCurrency::SEK,
            WasmFiatCurrency::CHF => FiatCurrency::CHF,
            WasmFiatCurrency::THB => FiatCurrency::THB,
            WasmFiatCurrency::TTD => FiatCurrency::TTD,
            WasmFiatCurrency::TND => FiatCurrency::TND,
            WasmFiatCurrency::TRY => FiatCurrency::TRY,
            WasmFiatCurrency::UGX => FiatCurrency::UGX,
            WasmFiatCurrency::UAH => FiatCurrency::UAH,
            WasmFiatCurrency::AED => FiatCurrency::AED,
            WasmFiatCurrency::USD => FiatCurrency::USD,
            WasmFiatCurrency::UYU => FiatCurrency::UYU,
            WasmFiatCurrency::UZS => FiatCurrency::UZS,
            WasmFiatCurrency::VND => FiatCurrency::VND,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

// We need this wrapper because, tsify doesn't support intoJs in async fns
#[wasm_bindgen(getter_with_clone)]
pub struct WasmUserSettingsData(pub WasmUserSettings);

#[wasm_bindgen]
pub struct WasmSettingsClient(SettingsClient);

impl From<SettingsClient> for WasmSettingsClient {
    fn from(value: SettingsClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmSettingsClient {
    #[wasm_bindgen(js_name = "getUserSettings")]
    pub async fn get_user_settings(&self) -> Result<WasmUserSettingsData, WasmError> {
        self.0
            .get_user_settings()
            .await
            .map_err(|e| e.into())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setBitcoinUnit")]
    pub async fn bitcoin_unit(&self, symbol: WasmBitcoinUnit) -> Result<WasmUserSettingsData, WasmError> {
        self.0
            .bitcoin_unit(symbol.into())
            .await
            .map_err(|e| e.into())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setFiatCurrency")]
    pub async fn fiat_currency(&self, symbol: WasmFiatCurrency) -> Result<WasmUserSettingsData, WasmError> {
        self.0
            .fiat_currency(symbol.into())
            .await
            .map_err(|e| e.into())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setTwoFaThreshold")]
    pub async fn two_fa_threshold(&self, amount: u64) -> Result<WasmUserSettingsData, WasmError> {
        self.0
            .two_fa_threshold(amount)
            .await
            .map_err(|e| e.into())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setHideEmptyUsedAddresses")]
    pub async fn hide_empty_used_addresses(
        &self,
        hide_empty_used_addresses: bool,
    ) -> Result<WasmUserSettingsData, WasmError> {
        self.0
            .hide_empty_used_addresses(hide_empty_used_addresses)
            .await
            .map_err(|e| e.into())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }
}
