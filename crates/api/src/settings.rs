use core::fmt;
use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
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
    pub TwoFactorAmountThreshold: Option<u64>,
    pub ReceiveInviterNotification: Option<u8>,
    pub ReceiveEmailIntegrationNotification: Option<u8>,
    pub WalletCreated: Option<u8>,
    pub AcceptTermsAndConditions: Option<u8>,
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

impl ApiClient for SettingsClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }
}

impl SettingsClient {
    pub async fn get_user_settings(&self) -> Result<UserSettings, Error> {
        let request = self.get("settings");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn update_bitcoin_unit(&self, symbol: BitcoinUnit) -> Result<UserSettings, Error> {
        let request = self
            .put("settings/currency/bitcoin")
            .body_json(UpdateBitcoinUnitRequestBody { Symbol: symbol })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn update_fiat_currency(&self, symbol: FiatCurrencySymbol) -> Result<UserSettings, Error> {
        let request = self
            .put("settings/currency/fiat")
            .body_json(UpdateFiatCurrencyRequestBody { Symbol: symbol })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn update_two_fa_threshold(&self, amount: u64) -> Result<UserSettings, Error> {
        let request = self
            .put("settings/2fa/threshold")
            .body_json(Update2FAThresholdRequestBody {
                TwoFactorAmountThreshold: amount,
            })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn update_hide_empty_used_addresses(
        &self,
        hide_empty_used_addresses: bool,
    ) -> Result<UserSettings, Error> {
        let request = self
            .put("settings/addresses/used/hide")
            .body_json(UpdateHideEmptyUsedAddressesRequestBody {
                HideEmptyUsedAddresses: hide_empty_used_addresses.into(),
            })?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn accept_terms_and_conditions(&self) -> Result<UserSettings, Error> {
        let request = self.put("settings/terms-and-conditions/accept");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }
}

#[cfg(test)]
mod tests {
    use andromeda_common::BitcoinUnit;

    use super::SettingsClient;
    use crate::{core::ApiClient, settings::FiatCurrencySymbol, tests::utils::common_api_client};

    #[tokio::test]
    #[ignore]
    async fn should_get_user_settings() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.get_user_settings().await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_bitcoin_unit() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.update_bitcoin_unit(BitcoinUnit::BTC).await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_fiat_currency() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.update_fiat_currency(FiatCurrencySymbol::USD).await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_two_fa_threshold() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.update_two_fa_threshold(1000).await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_hide_empty_used_addresses() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client.update_hide_empty_used_addresses(true).await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }
}
