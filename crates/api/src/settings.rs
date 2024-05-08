use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Session};
use serde::{Deserialize, Serialize};

use crate::{error::Error, proton_response_ext::ProtonResponseExt, BASE_WALLET_API_V1};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum FiatCurrency {
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

impl ToString for FiatCurrency {
    fn to_string(&self) -> String {
        match self {
            FiatCurrency::ALL => String::from("ALL"),
            FiatCurrency::DZD => String::from("DZD"),
            FiatCurrency::ARS => String::from("ARS"),
            FiatCurrency::AMD => String::from("AMD"),
            FiatCurrency::AUD => String::from("AUD"),
            FiatCurrency::AZN => String::from("AZN"),
            FiatCurrency::BHD => String::from("BHD"),
            FiatCurrency::BDT => String::from("BDT"),
            FiatCurrency::BYN => String::from("BYN"),
            FiatCurrency::BMD => String::from("BMD"),
            FiatCurrency::BOB => String::from("BOB"),
            FiatCurrency::BAM => String::from("BAM"),
            FiatCurrency::BRL => String::from("BRL"),
            FiatCurrency::BGN => String::from("BGN"),
            FiatCurrency::KHR => String::from("KHR"),
            FiatCurrency::CAD => String::from("CAD"),
            FiatCurrency::CLP => String::from("CLP"),
            FiatCurrency::CNY => String::from("CNY"),
            FiatCurrency::COP => String::from("COP"),
            FiatCurrency::CRC => String::from("CRC"),
            FiatCurrency::HRK => String::from("HRK"),
            FiatCurrency::CUP => String::from("CUP"),
            FiatCurrency::CZK => String::from("CZK"),
            FiatCurrency::DKK => String::from("DKK"),
            FiatCurrency::DOP => String::from("DOP"),
            FiatCurrency::EGP => String::from("EGP"),
            FiatCurrency::EUR => String::from("EUR"),
            FiatCurrency::GEL => String::from("GEL"),
            FiatCurrency::GHS => String::from("GHS"),
            FiatCurrency::GTQ => String::from("GTQ"),
            FiatCurrency::HNL => String::from("HNL"),
            FiatCurrency::HKD => String::from("HKD"),
            FiatCurrency::HUF => String::from("HUF"),
            FiatCurrency::ISK => String::from("ISK"),
            FiatCurrency::INR => String::from("INR"),
            FiatCurrency::IDR => String::from("IDR"),
            FiatCurrency::IRR => String::from("IRR"),
            FiatCurrency::IQD => String::from("IQD"),
            FiatCurrency::ILS => String::from("ILS"),
            FiatCurrency::JMD => String::from("JMD"),
            FiatCurrency::JPY => String::from("JPY"),
            FiatCurrency::JOD => String::from("JOD"),
            FiatCurrency::KZT => String::from("KZT"),
            FiatCurrency::KES => String::from("KES"),
            FiatCurrency::KWD => String::from("KWD"),
            FiatCurrency::KGS => String::from("KGS"),
            FiatCurrency::LBP => String::from("LBP"),
            FiatCurrency::MKD => String::from("MKD"),
            FiatCurrency::MYR => String::from("MYR"),
            FiatCurrency::MUR => String::from("MUR"),
            FiatCurrency::MXN => String::from("MXN"),
            FiatCurrency::MDL => String::from("MDL"),
            FiatCurrency::MNT => String::from("MNT"),
            FiatCurrency::MAD => String::from("MAD"),
            FiatCurrency::MMK => String::from("MMK"),
            FiatCurrency::NAD => String::from("NAD"),
            FiatCurrency::NPR => String::from("NPR"),
            FiatCurrency::TWD => String::from("TWD"),
            FiatCurrency::NZD => String::from("NZD"),
            FiatCurrency::NIO => String::from("NIO"),
            FiatCurrency::NGN => String::from("NGN"),
            FiatCurrency::NOK => String::from("NOK"),
            FiatCurrency::OMR => String::from("OMR"),
            FiatCurrency::PKR => String::from("PKR"),
            FiatCurrency::PAB => String::from("PAB"),
            FiatCurrency::PEN => String::from("PEN"),
            FiatCurrency::PHP => String::from("PHP"),
            FiatCurrency::PLN => String::from("PLN"),
            FiatCurrency::GBP => String::from("GBP"),
            FiatCurrency::QAR => String::from("QAR"),
            FiatCurrency::RON => String::from("RON"),
            FiatCurrency::RUB => String::from("RUB"),
            FiatCurrency::SAR => String::from("SAR"),
            FiatCurrency::RSD => String::from("RSD"),
            FiatCurrency::SGD => String::from("SGD"),
            FiatCurrency::ZAR => String::from("ZAR"),
            FiatCurrency::KRW => String::from("KRW"),
            FiatCurrency::SSP => String::from("SSP"),
            FiatCurrency::VES => String::from("VES"),
            FiatCurrency::LKR => String::from("LKR"),
            FiatCurrency::SEK => String::from("SEK"),
            FiatCurrency::CHF => String::from("CHF"),
            FiatCurrency::THB => String::from("THB"),
            FiatCurrency::TTD => String::from("TTD"),
            FiatCurrency::TND => String::from("TND"),
            FiatCurrency::TRY => String::from("TRY"),
            FiatCurrency::UGX => String::from("UGX"),
            FiatCurrency::UAH => String::from("UAH"),
            FiatCurrency::AED => String::from("AED"),
            FiatCurrency::USD => String::from("USD"),
            FiatCurrency::UYU => String::from("UYU"),
            FiatCurrency::UZS => String::from("UZS"),
            FiatCurrency::VND => String::from("VND"),
        }
    }
}

#[derive(Clone)]
pub struct SettingsClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct UserSettings {
    pub BitcoinUnit: BitcoinUnit,
    pub FiatCurrency: FiatCurrency,
    pub HideEmptyUsedAddresses: u8,
    pub ShowWalletRecovery: u8,
    pub TwoFactorAmountThreshold: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetUserSettingsResponseBody {
    //TODO:: code need to be used. remove all #[allow(dead_code)]
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletUserSettings: UserSettings,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct UpdateBitcoinUnitRequestBody {
    pub Symbol: BitcoinUnit,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct UpdateFiatCurrencyRequestBody {
    pub Symbol: FiatCurrency,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct Update2FAThresholdRequestBody {
    pub TwoFactorAmountThreshold: u64,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct UpdateHideEmptyUsedAddressesRequestBody {
    pub HideEmptyUsedAddresses: u8,
}

impl SettingsClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_user_settings(&self) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/settings", BASE_WALLET_API_V1));
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn bitcoin_unit(&self, symbol: BitcoinUnit) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/currency/bitcoin", BASE_WALLET_API_V1))
            .json_body(UpdateBitcoinUnitRequestBody { Symbol: symbol })?;
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn fiat_currency(&self, symbol: FiatCurrency) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/currency/fiat", BASE_WALLET_API_V1))
            .json_body(UpdateFiatCurrencyRequestBody { Symbol: symbol })?;
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn two_fa_threshold(&self, amount: u64) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/2fa/threshold", BASE_WALLET_API_V1))
            .json_body(Update2FAThresholdRequestBody {
                TwoFactorAmountThreshold: amount,
            })?;
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn hide_empty_used_addresses(&self, hide_empty_used_addresses: bool) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(
            Method::PUT,
            format!("{}/settings/addresses/used/hide", BASE_WALLET_API_V1),
        )
        .json_body(UpdateHideEmptyUsedAddressesRequestBody {
            HideEmptyUsedAddresses: hide_empty_used_addresses.into(),
        })?;
        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let session = common_session().await;
        let client = SettingsClient::new(session);

        let settings = client.get_user_settings().await;
        println!("request done: {:?}", settings);
    }
}
