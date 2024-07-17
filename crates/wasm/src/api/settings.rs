use andromeda_api::settings::{FiatCurrencySymbol, SettingsClient, UserReceiveNotificationEmailTypes, UserSettings};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::{error::ErrorExt, types::WasmBitcoinUnit};

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmUserReceiveNotificationEmailTypes {
    NotificationToInviter,
    EmailIntegration,
    TransactionalBvE,
    Unsupported,
}

impl From<UserReceiveNotificationEmailTypes> for WasmUserReceiveNotificationEmailTypes {
    fn from(value: UserReceiveNotificationEmailTypes) -> Self {
        match value {
            UserReceiveNotificationEmailTypes::NotificationToInviter => {
                WasmUserReceiveNotificationEmailTypes::NotificationToInviter
            }
            UserReceiveNotificationEmailTypes::EmailIntegration => {
                WasmUserReceiveNotificationEmailTypes::EmailIntegration
            }
            UserReceiveNotificationEmailTypes::TransactionalBvE => {
                WasmUserReceiveNotificationEmailTypes::TransactionalBvE
            }
            UserReceiveNotificationEmailTypes::Unsupported => WasmUserReceiveNotificationEmailTypes::Unsupported,
        }
    }
}

impl From<WasmUserReceiveNotificationEmailTypes> for UserReceiveNotificationEmailTypes {
    fn from(value: WasmUserReceiveNotificationEmailTypes) -> Self {
        match value {
            WasmUserReceiveNotificationEmailTypes::NotificationToInviter => {
                UserReceiveNotificationEmailTypes::NotificationToInviter
            }
            WasmUserReceiveNotificationEmailTypes::EmailIntegration => {
                UserReceiveNotificationEmailTypes::EmailIntegration
            }
            WasmUserReceiveNotificationEmailTypes::TransactionalBvE => {
                UserReceiveNotificationEmailTypes::TransactionalBvE
            }
            WasmUserReceiveNotificationEmailTypes::Unsupported => UserReceiveNotificationEmailTypes::Unsupported,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmFiatCurrencySymbol {
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

impl From<FiatCurrencySymbol> for WasmFiatCurrencySymbol {
    fn from(value: FiatCurrencySymbol) -> Self {
        match value {
            FiatCurrencySymbol::ALL => WasmFiatCurrencySymbol::ALL,
            FiatCurrencySymbol::DZD => WasmFiatCurrencySymbol::DZD,
            FiatCurrencySymbol::ARS => WasmFiatCurrencySymbol::ARS,
            FiatCurrencySymbol::AMD => WasmFiatCurrencySymbol::AMD,
            FiatCurrencySymbol::AUD => WasmFiatCurrencySymbol::AUD,
            FiatCurrencySymbol::AZN => WasmFiatCurrencySymbol::AZN,
            FiatCurrencySymbol::BHD => WasmFiatCurrencySymbol::BHD,
            FiatCurrencySymbol::BDT => WasmFiatCurrencySymbol::BDT,
            FiatCurrencySymbol::BYN => WasmFiatCurrencySymbol::BYN,
            FiatCurrencySymbol::BMD => WasmFiatCurrencySymbol::BMD,
            FiatCurrencySymbol::BOB => WasmFiatCurrencySymbol::BOB,
            FiatCurrencySymbol::BAM => WasmFiatCurrencySymbol::BAM,
            FiatCurrencySymbol::BRL => WasmFiatCurrencySymbol::BRL,
            FiatCurrencySymbol::BGN => WasmFiatCurrencySymbol::BGN,
            FiatCurrencySymbol::KHR => WasmFiatCurrencySymbol::KHR,
            FiatCurrencySymbol::CAD => WasmFiatCurrencySymbol::CAD,
            FiatCurrencySymbol::CLP => WasmFiatCurrencySymbol::CLP,
            FiatCurrencySymbol::CNY => WasmFiatCurrencySymbol::CNY,
            FiatCurrencySymbol::COP => WasmFiatCurrencySymbol::COP,
            FiatCurrencySymbol::CRC => WasmFiatCurrencySymbol::CRC,
            FiatCurrencySymbol::HRK => WasmFiatCurrencySymbol::HRK,
            FiatCurrencySymbol::CUP => WasmFiatCurrencySymbol::CUP,
            FiatCurrencySymbol::CZK => WasmFiatCurrencySymbol::CZK,
            FiatCurrencySymbol::DKK => WasmFiatCurrencySymbol::DKK,
            FiatCurrencySymbol::DOP => WasmFiatCurrencySymbol::DOP,
            FiatCurrencySymbol::EGP => WasmFiatCurrencySymbol::EGP,
            FiatCurrencySymbol::EUR => WasmFiatCurrencySymbol::EUR,
            FiatCurrencySymbol::GEL => WasmFiatCurrencySymbol::GEL,
            FiatCurrencySymbol::GHS => WasmFiatCurrencySymbol::GHS,
            FiatCurrencySymbol::GTQ => WasmFiatCurrencySymbol::GTQ,
            FiatCurrencySymbol::HNL => WasmFiatCurrencySymbol::HNL,
            FiatCurrencySymbol::HKD => WasmFiatCurrencySymbol::HKD,
            FiatCurrencySymbol::HUF => WasmFiatCurrencySymbol::HUF,
            FiatCurrencySymbol::ISK => WasmFiatCurrencySymbol::ISK,
            FiatCurrencySymbol::INR => WasmFiatCurrencySymbol::INR,
            FiatCurrencySymbol::IDR => WasmFiatCurrencySymbol::IDR,
            FiatCurrencySymbol::IRR => WasmFiatCurrencySymbol::IRR,
            FiatCurrencySymbol::IQD => WasmFiatCurrencySymbol::IQD,
            FiatCurrencySymbol::ILS => WasmFiatCurrencySymbol::ILS,
            FiatCurrencySymbol::JMD => WasmFiatCurrencySymbol::JMD,
            FiatCurrencySymbol::JPY => WasmFiatCurrencySymbol::JPY,
            FiatCurrencySymbol::JOD => WasmFiatCurrencySymbol::JOD,
            FiatCurrencySymbol::KZT => WasmFiatCurrencySymbol::KZT,
            FiatCurrencySymbol::KES => WasmFiatCurrencySymbol::KES,
            FiatCurrencySymbol::KWD => WasmFiatCurrencySymbol::KWD,
            FiatCurrencySymbol::KGS => WasmFiatCurrencySymbol::KGS,
            FiatCurrencySymbol::LBP => WasmFiatCurrencySymbol::LBP,
            FiatCurrencySymbol::MKD => WasmFiatCurrencySymbol::MKD,
            FiatCurrencySymbol::MYR => WasmFiatCurrencySymbol::MYR,
            FiatCurrencySymbol::MUR => WasmFiatCurrencySymbol::MUR,
            FiatCurrencySymbol::MXN => WasmFiatCurrencySymbol::MXN,
            FiatCurrencySymbol::MDL => WasmFiatCurrencySymbol::MDL,
            FiatCurrencySymbol::MNT => WasmFiatCurrencySymbol::MNT,
            FiatCurrencySymbol::MAD => WasmFiatCurrencySymbol::MAD,
            FiatCurrencySymbol::MMK => WasmFiatCurrencySymbol::MMK,
            FiatCurrencySymbol::NAD => WasmFiatCurrencySymbol::NAD,
            FiatCurrencySymbol::NPR => WasmFiatCurrencySymbol::NPR,
            FiatCurrencySymbol::TWD => WasmFiatCurrencySymbol::TWD,
            FiatCurrencySymbol::NZD => WasmFiatCurrencySymbol::NZD,
            FiatCurrencySymbol::NIO => WasmFiatCurrencySymbol::NIO,
            FiatCurrencySymbol::NGN => WasmFiatCurrencySymbol::NGN,
            FiatCurrencySymbol::NOK => WasmFiatCurrencySymbol::NOK,
            FiatCurrencySymbol::OMR => WasmFiatCurrencySymbol::OMR,
            FiatCurrencySymbol::PKR => WasmFiatCurrencySymbol::PKR,
            FiatCurrencySymbol::PAB => WasmFiatCurrencySymbol::PAB,
            FiatCurrencySymbol::PEN => WasmFiatCurrencySymbol::PEN,
            FiatCurrencySymbol::PHP => WasmFiatCurrencySymbol::PHP,
            FiatCurrencySymbol::PLN => WasmFiatCurrencySymbol::PLN,
            FiatCurrencySymbol::GBP => WasmFiatCurrencySymbol::GBP,
            FiatCurrencySymbol::QAR => WasmFiatCurrencySymbol::QAR,
            FiatCurrencySymbol::RON => WasmFiatCurrencySymbol::RON,
            FiatCurrencySymbol::RUB => WasmFiatCurrencySymbol::RUB,
            FiatCurrencySymbol::SAR => WasmFiatCurrencySymbol::SAR,
            FiatCurrencySymbol::RSD => WasmFiatCurrencySymbol::RSD,
            FiatCurrencySymbol::SGD => WasmFiatCurrencySymbol::SGD,
            FiatCurrencySymbol::ZAR => WasmFiatCurrencySymbol::ZAR,
            FiatCurrencySymbol::KRW => WasmFiatCurrencySymbol::KRW,
            FiatCurrencySymbol::SSP => WasmFiatCurrencySymbol::SSP,
            FiatCurrencySymbol::VES => WasmFiatCurrencySymbol::VES,
            FiatCurrencySymbol::LKR => WasmFiatCurrencySymbol::LKR,
            FiatCurrencySymbol::SEK => WasmFiatCurrencySymbol::SEK,
            FiatCurrencySymbol::CHF => WasmFiatCurrencySymbol::CHF,
            FiatCurrencySymbol::THB => WasmFiatCurrencySymbol::THB,
            FiatCurrencySymbol::TTD => WasmFiatCurrencySymbol::TTD,
            FiatCurrencySymbol::TND => WasmFiatCurrencySymbol::TND,
            FiatCurrencySymbol::TRY => WasmFiatCurrencySymbol::TRY,
            FiatCurrencySymbol::UGX => WasmFiatCurrencySymbol::UGX,
            FiatCurrencySymbol::UAH => WasmFiatCurrencySymbol::UAH,
            FiatCurrencySymbol::AED => WasmFiatCurrencySymbol::AED,
            FiatCurrencySymbol::USD => WasmFiatCurrencySymbol::USD,
            FiatCurrencySymbol::UYU => WasmFiatCurrencySymbol::UYU,
            FiatCurrencySymbol::UZS => WasmFiatCurrencySymbol::UZS,
            FiatCurrencySymbol::VND => WasmFiatCurrencySymbol::VND,
        }
    }
}

impl From<WasmFiatCurrencySymbol> for FiatCurrencySymbol {
    fn from(value: WasmFiatCurrencySymbol) -> Self {
        match value {
            WasmFiatCurrencySymbol::ALL => FiatCurrencySymbol::ALL,
            WasmFiatCurrencySymbol::DZD => FiatCurrencySymbol::DZD,
            WasmFiatCurrencySymbol::ARS => FiatCurrencySymbol::ARS,
            WasmFiatCurrencySymbol::AMD => FiatCurrencySymbol::AMD,
            WasmFiatCurrencySymbol::AUD => FiatCurrencySymbol::AUD,
            WasmFiatCurrencySymbol::AZN => FiatCurrencySymbol::AZN,
            WasmFiatCurrencySymbol::BHD => FiatCurrencySymbol::BHD,
            WasmFiatCurrencySymbol::BDT => FiatCurrencySymbol::BDT,
            WasmFiatCurrencySymbol::BYN => FiatCurrencySymbol::BYN,
            WasmFiatCurrencySymbol::BMD => FiatCurrencySymbol::BMD,
            WasmFiatCurrencySymbol::BOB => FiatCurrencySymbol::BOB,
            WasmFiatCurrencySymbol::BAM => FiatCurrencySymbol::BAM,
            WasmFiatCurrencySymbol::BRL => FiatCurrencySymbol::BRL,
            WasmFiatCurrencySymbol::BGN => FiatCurrencySymbol::BGN,
            WasmFiatCurrencySymbol::KHR => FiatCurrencySymbol::KHR,
            WasmFiatCurrencySymbol::CAD => FiatCurrencySymbol::CAD,
            WasmFiatCurrencySymbol::CLP => FiatCurrencySymbol::CLP,
            WasmFiatCurrencySymbol::CNY => FiatCurrencySymbol::CNY,
            WasmFiatCurrencySymbol::COP => FiatCurrencySymbol::COP,
            WasmFiatCurrencySymbol::CRC => FiatCurrencySymbol::CRC,
            WasmFiatCurrencySymbol::HRK => FiatCurrencySymbol::HRK,
            WasmFiatCurrencySymbol::CUP => FiatCurrencySymbol::CUP,
            WasmFiatCurrencySymbol::CZK => FiatCurrencySymbol::CZK,
            WasmFiatCurrencySymbol::DKK => FiatCurrencySymbol::DKK,
            WasmFiatCurrencySymbol::DOP => FiatCurrencySymbol::DOP,
            WasmFiatCurrencySymbol::EGP => FiatCurrencySymbol::EGP,
            WasmFiatCurrencySymbol::EUR => FiatCurrencySymbol::EUR,
            WasmFiatCurrencySymbol::GEL => FiatCurrencySymbol::GEL,
            WasmFiatCurrencySymbol::GHS => FiatCurrencySymbol::GHS,
            WasmFiatCurrencySymbol::GTQ => FiatCurrencySymbol::GTQ,
            WasmFiatCurrencySymbol::HNL => FiatCurrencySymbol::HNL,
            WasmFiatCurrencySymbol::HKD => FiatCurrencySymbol::HKD,
            WasmFiatCurrencySymbol::HUF => FiatCurrencySymbol::HUF,
            WasmFiatCurrencySymbol::ISK => FiatCurrencySymbol::ISK,
            WasmFiatCurrencySymbol::INR => FiatCurrencySymbol::INR,
            WasmFiatCurrencySymbol::IDR => FiatCurrencySymbol::IDR,
            WasmFiatCurrencySymbol::IRR => FiatCurrencySymbol::IRR,
            WasmFiatCurrencySymbol::IQD => FiatCurrencySymbol::IQD,
            WasmFiatCurrencySymbol::ILS => FiatCurrencySymbol::ILS,
            WasmFiatCurrencySymbol::JMD => FiatCurrencySymbol::JMD,
            WasmFiatCurrencySymbol::JPY => FiatCurrencySymbol::JPY,
            WasmFiatCurrencySymbol::JOD => FiatCurrencySymbol::JOD,
            WasmFiatCurrencySymbol::KZT => FiatCurrencySymbol::KZT,
            WasmFiatCurrencySymbol::KES => FiatCurrencySymbol::KES,
            WasmFiatCurrencySymbol::KWD => FiatCurrencySymbol::KWD,
            WasmFiatCurrencySymbol::KGS => FiatCurrencySymbol::KGS,
            WasmFiatCurrencySymbol::LBP => FiatCurrencySymbol::LBP,
            WasmFiatCurrencySymbol::MKD => FiatCurrencySymbol::MKD,
            WasmFiatCurrencySymbol::MYR => FiatCurrencySymbol::MYR,
            WasmFiatCurrencySymbol::MUR => FiatCurrencySymbol::MUR,
            WasmFiatCurrencySymbol::MXN => FiatCurrencySymbol::MXN,
            WasmFiatCurrencySymbol::MDL => FiatCurrencySymbol::MDL,
            WasmFiatCurrencySymbol::MNT => FiatCurrencySymbol::MNT,
            WasmFiatCurrencySymbol::MAD => FiatCurrencySymbol::MAD,
            WasmFiatCurrencySymbol::MMK => FiatCurrencySymbol::MMK,
            WasmFiatCurrencySymbol::NAD => FiatCurrencySymbol::NAD,
            WasmFiatCurrencySymbol::NPR => FiatCurrencySymbol::NPR,
            WasmFiatCurrencySymbol::TWD => FiatCurrencySymbol::TWD,
            WasmFiatCurrencySymbol::NZD => FiatCurrencySymbol::NZD,
            WasmFiatCurrencySymbol::NIO => FiatCurrencySymbol::NIO,
            WasmFiatCurrencySymbol::NGN => FiatCurrencySymbol::NGN,
            WasmFiatCurrencySymbol::NOK => FiatCurrencySymbol::NOK,
            WasmFiatCurrencySymbol::OMR => FiatCurrencySymbol::OMR,
            WasmFiatCurrencySymbol::PKR => FiatCurrencySymbol::PKR,
            WasmFiatCurrencySymbol::PAB => FiatCurrencySymbol::PAB,
            WasmFiatCurrencySymbol::PEN => FiatCurrencySymbol::PEN,
            WasmFiatCurrencySymbol::PHP => FiatCurrencySymbol::PHP,
            WasmFiatCurrencySymbol::PLN => FiatCurrencySymbol::PLN,
            WasmFiatCurrencySymbol::GBP => FiatCurrencySymbol::GBP,
            WasmFiatCurrencySymbol::QAR => FiatCurrencySymbol::QAR,
            WasmFiatCurrencySymbol::RON => FiatCurrencySymbol::RON,
            WasmFiatCurrencySymbol::RUB => FiatCurrencySymbol::RUB,
            WasmFiatCurrencySymbol::SAR => FiatCurrencySymbol::SAR,
            WasmFiatCurrencySymbol::RSD => FiatCurrencySymbol::RSD,
            WasmFiatCurrencySymbol::SGD => FiatCurrencySymbol::SGD,
            WasmFiatCurrencySymbol::ZAR => FiatCurrencySymbol::ZAR,
            WasmFiatCurrencySymbol::KRW => FiatCurrencySymbol::KRW,
            WasmFiatCurrencySymbol::SSP => FiatCurrencySymbol::SSP,
            WasmFiatCurrencySymbol::VES => FiatCurrencySymbol::VES,
            WasmFiatCurrencySymbol::LKR => FiatCurrencySymbol::LKR,
            WasmFiatCurrencySymbol::SEK => FiatCurrencySymbol::SEK,
            WasmFiatCurrencySymbol::CHF => FiatCurrencySymbol::CHF,
            WasmFiatCurrencySymbol::THB => FiatCurrencySymbol::THB,
            WasmFiatCurrencySymbol::TTD => FiatCurrencySymbol::TTD,
            WasmFiatCurrencySymbol::TND => FiatCurrencySymbol::TND,
            WasmFiatCurrencySymbol::TRY => FiatCurrencySymbol::TRY,
            WasmFiatCurrencySymbol::UGX => FiatCurrencySymbol::UGX,
            WasmFiatCurrencySymbol::UAH => FiatCurrencySymbol::UAH,
            WasmFiatCurrencySymbol::AED => FiatCurrencySymbol::AED,
            WasmFiatCurrencySymbol::USD => FiatCurrencySymbol::USD,
            WasmFiatCurrencySymbol::UYU => FiatCurrencySymbol::UYU,
            WasmFiatCurrencySymbol::UZS => FiatCurrencySymbol::UZS,
            WasmFiatCurrencySymbol::VND => FiatCurrencySymbol::VND,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmUserSettings {
    pub BitcoinUnit: WasmBitcoinUnit,
    pub FiatCurrency: WasmFiatCurrencySymbol,
    pub HideEmptyUsedAddresses: u8,
    pub TwoFactorAmountThreshold: Option<u64>,
    pub ReceiveInviterNotification: Option<u8>,
    pub ReceiveEmailIntegrationNotification: Option<u8>,
    pub WalletCreated: Option<u8>,
    pub AcceptTermsAndConditions: Option<u8>,
}

impl From<UserSettings> for WasmUserSettings {
    fn from(value: UserSettings) -> Self {
        Self {
            BitcoinUnit: value.BitcoinUnit.into(),
            FiatCurrency: value.FiatCurrency.into(),
            HideEmptyUsedAddresses: value.HideEmptyUsedAddresses,
            TwoFactorAmountThreshold: value.TwoFactorAmountThreshold,
            ReceiveInviterNotification: value.ReceiveInviterNotification,
            ReceiveEmailIntegrationNotification: value.ReceiveEmailIntegrationNotification,
            WalletCreated: value.WalletCreated,
            AcceptTermsAndConditions: value.AcceptTermsAndConditions,
        }
    }
}

// We need this wrapper because, tsify doesn't support intoJs in async fns
#[wasm_bindgen(getter_with_clone)]
pub struct WasmUserSettingsData(pub WasmUserSettings);

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmSettingsClient(SettingsClient);

impl From<SettingsClient> for WasmSettingsClient {
    fn from(value: SettingsClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmSettingsClient {
    #[wasm_bindgen(js_name = "getUserSettings")]
    pub async fn get_user_settings(&self) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .get_user_settings()
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setBitcoinUnit")]
    pub async fn bitcoin_unit(&self, symbol: WasmBitcoinUnit) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .update_bitcoin_unit(symbol.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setFiatCurrency")]
    pub async fn fiat_currency(&self, symbol: WasmFiatCurrencySymbol) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .update_fiat_currency(symbol.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setTwoFaThreshold")]
    pub async fn two_fa_threshold(&self, amount: u64) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .update_two_fa_threshold(amount)
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setHideEmptyUsedAddresses")]
    pub async fn hide_empty_used_addresses(
        &self,
        hide_empty_used_addresses: bool,
    ) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .update_hide_empty_used_addresses(hide_empty_used_addresses)
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "setReceiveNotificationEmail")]
    pub async fn receive_notification_email(
        &self,
        email_type: WasmUserReceiveNotificationEmailTypes,
        is_enable: bool,
    ) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .update_receive_notification_email(email_type.into(), is_enable)
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }

    #[wasm_bindgen(js_name = "acceptTermsAndConditions")]
    pub async fn accept_terms_and_conditions(&self) -> Result<WasmUserSettingsData, JsValue> {
        self.0
            .accept_terms_and_conditions()
            .await
            .map_err(|e| e.to_js_error())
            .map(|settings| WasmUserSettingsData(settings.into()))
    }
}
