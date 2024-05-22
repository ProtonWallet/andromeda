use core::fmt;
use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use serde::{Deserialize, Serialize};

use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum FiatCurrencySymbol {
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

impl fmt::Display for FiatCurrencySymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct UserSettings {
    pub BitcoinUnit: BitcoinUnit,
    pub FiatCurrency: FiatCurrencySymbol,
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
    pub Symbol: FiatCurrencySymbol,
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

#[derive(Clone)]
pub struct SettingsClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl SettingsClient {
    pub fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn get_user_settings(&self) -> Result<UserSettings, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "settings")
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn bitcoin_unit(&self, symbol: BitcoinUnit) -> Result<UserSettings, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "settings/currency/bitcoin")
            .to_put_request()
            .json_body(UpdateBitcoinUnitRequestBody { Symbol: symbol })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn fiat_currency(&self, symbol: FiatCurrencySymbol) -> Result<UserSettings, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "settings/currency/fiat")
            .to_put_request()
            .json_body(UpdateFiatCurrencyRequestBody { Symbol: symbol })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn two_fa_threshold(&self, amount: u64) -> Result<UserSettings, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "settings/2fa/threshold")
            .to_put_request()
            .json_body(Update2FAThresholdRequestBody {
                TwoFactorAmountThreshold: amount,
            })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn hide_empty_used_addresses(&self, hide_empty_used_addresses: bool) -> Result<UserSettings, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "settings")
            .to_put_request()
            .json_body(UpdateHideEmptyUsedAddressesRequestBody {
                HideEmptyUsedAddresses: hide_empty_used_addresses.into(),
            })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsClient;
    use crate::tests::utils::common_api_client;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.get_user_settings().await;
        println!("request done: {:?}", settings);
    }
}
