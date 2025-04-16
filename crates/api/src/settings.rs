use core::fmt;
use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum UserReceiveNotificationEmailTypes {
    NotificationToInviter = 1,
    EmailIntegration = 2,
    TransactionalBvE = 4,
    #[serde(other)]
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Default)]
#[allow(clippy::upper_case_acronyms)]
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
    #[default]
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
    /// Flag if user had accepted terms and conditions, 0: no, 1: yes
    pub AcceptTermsAndConditions: Option<u8>,
    /// Tell the client that it is allowed to show the review page, 0: no, 1: yes
    pub AllowReview: Option<u8>,
    pub BitcoinUnit: BitcoinUnit,
    pub FiatCurrency: FiatCurrencySymbol,
    /// Hide empty used addresses, 0: disabled, 1: enabled
    pub HideEmptyUsedAddresses: u8,
    /// Ask for 2FA verification when an amount threshold is reached
    pub TwoFactorAmountThreshold: Option<u64>,
    /// Receive inviter notification, 0: disabled, 1: enabled
    pub ReceiveInviterNotification: Option<u8>,
    /// Receive email integration notification, 0: disabled, 1: enabled
    pub ReceiveEmailIntegrationNotification: Option<u8>,
    /// Receive transaction notification, 0: disabled, 1: enabled
    pub ReceiveTransactionNotification: Option<u8>,
    /// User has already created a wallet once, 0: no, 1: yes
    pub WalletCreated: Option<u8>,
    /// Timestamp about when user saw the review page on client
    pub ReviewTime: Option<u64>,
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

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct UpdateReceiveNotificationEmailRequestBody {
    pub EmailType: UserReceiveNotificationEmailTypes,
    pub IsEnabled: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetUserWalletEligibilityResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub IsEligible: u8,
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

    pub async fn update_receive_notification_email(
        &self,
        email_type: UserReceiveNotificationEmailTypes,
        is_enable: bool,
    ) -> Result<UserSettings, Error> {
        let request = self
            .put("settings/notification-email")
            .body_json(UpdateReceiveNotificationEmailRequestBody {
                EmailType: email_type,
                IsEnabled: is_enable.into(),
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

    pub async fn update_review_time(&self) -> Result<UserSettings, Error> {
        let request = self.put("settings/review-time");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserSettingsResponseBody>()?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn get_user_wallet_eligibility(&self) -> Result<u8, Error> {
        let request = self.get("settings/eligible");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetUserWalletEligibilityResponseBody>()?;

        Ok(parsed.IsEligible)
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsClient;
    use crate::{
        core::ApiClient, settings::FiatCurrencySymbol, tests::utils::common_api_client,
        tests::utils::setup_test_connection, BASE_WALLET_API_V1,
    };
    use andromeda_common::BitcoinUnit;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

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

    #[tokio::test]
    #[ignore]
    async fn should_update_receive_notification_email() {
        let api_client = common_api_client().await;
        let client = SettingsClient::new(api_client);

        let settings = client
            .update_receive_notification_email(
                crate::settings::UserReceiveNotificationEmailTypes::NotificationToInviter,
                true,
            )
            .await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());

        let settings = client
            .update_receive_notification_email(
                crate::settings::UserReceiveNotificationEmailTypes::NotificationToInviter,
                false,
            )
            .await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());

        let settings = client
            .update_receive_notification_email(
                crate::settings::UserReceiveNotificationEmailTypes::EmailIntegration,
                true,
            )
            .await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());

        let settings = client
            .update_receive_notification_email(
                crate::settings::UserReceiveNotificationEmailTypes::EmailIntegration,
                false,
            )
            .await;
        println!("request done: {:?}", settings);
        assert!(settings.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_settings_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "CHF",
                    "HideEmptyUsedAddresses": 1,
                    "TwoFactorAmountThreshold": 1000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let result = client.get_user_settings().await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.BitcoinUnit, BitcoinUnit::BTC);
                assert_eq!(settings.FiatCurrency, FiatCurrencySymbol::CHF);
                assert_eq!(settings.HideEmptyUsedAddresses, 1);
                assert_eq!(settings.TwoFactorAmountThreshold.unwrap(), 1000);
                assert_eq!(settings.ReceiveInviterNotification.unwrap(), 1);
                assert_eq!(settings.ReceiveEmailIntegrationNotification.unwrap(), 1);
                assert_eq!(settings.ReceiveTransactionNotification.unwrap(), 1);
                assert_eq!(settings.WalletCreated.unwrap(), 1);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_bitcoin_unit_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "SATS",
                    "FiatCurrency": "CHF",
                    "HideEmptyUsedAddresses": 1,
                    "TwoFactorAmountThreshold": 1000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/currency/bitcoin", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "Symbol": "SATS",
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let bitcoin_unit = BitcoinUnit::SATS;
        let result = client.update_bitcoin_unit(bitcoin_unit).await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.BitcoinUnit, bitcoin_unit);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_fiat_currency_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 1,
                    "TwoFactorAmountThreshold": 1000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/currency/fiat", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "Symbol": "TWD",
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let currency = FiatCurrencySymbol::TWD;
        let result = client.update_fiat_currency(currency).await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.FiatCurrency, currency);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_two_fa_threshold_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 1,
                    "TwoFactorAmountThreshold": 3000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/2fa/threshold", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "TwoFactorAmountThreshold": 3000,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let amount = 3000;
        let result = client.update_two_fa_threshold(amount).await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.TwoFactorAmountThreshold.unwrap(), amount);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_hide_empty_used_addresses_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 0,
                    "TwoFactorAmountThreshold": 3000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/addresses/used/hide", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "HideEmptyUsedAddresses": 0,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let hide = false;
        let result = client.update_hide_empty_used_addresses(hide).await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.HideEmptyUsedAddresses, 0);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_receive_notification_email_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 1,
                    "TwoFactorAmountThreshold": 3000,
                    "ReceiveInviterNotification": 0,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/notification-email", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "EmailType": crate::settings::UserReceiveNotificationEmailTypes::NotificationToInviter,
                "IsEnabled": 0,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let result = client
            .update_receive_notification_email(
                crate::settings::UserReceiveNotificationEmailTypes::NotificationToInviter,
                false,
            )
            .await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.ReceiveInviterNotification.unwrap(), 0);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_accept_terms_and_conditions_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "AcceptTermsAndConditions": 1,
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 0,
                    "TwoFactorAmountThreshold": 3000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1
                }
            }
        );
        let req_path: String = format!("{}/settings/terms-and-conditions/accept", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let result = client.accept_terms_and_conditions().await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.AcceptTermsAndConditions.unwrap(), 1);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_review_time_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletUserSettings": {
                    "AcceptTermsAndConditions": 1,
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "TWD",
                    "HideEmptyUsedAddresses": 0,
                    "TwoFactorAmountThreshold": 3000,
                    "ReceiveInviterNotification": 1,
                    "ReceiveEmailIntegrationNotification": 1,
                    "ReceiveTransactionNotification": 1,
                    "WalletCreated": 1,
                    "ReviewTime": 12345678,
                }
            }
        );
        let req_path: String = format!("{}/settings/review-time", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let result = client.update_review_time().await;
        match result {
            Ok(settings) => {
                assert_eq!(settings.ReviewTime.unwrap(), 12345678);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_user_wallet_eligibility_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "IsEligible": 0,
            }
        );
        let req_path: String = format!("{}/settings/eligible", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = SettingsClient::new(api_client);
        let result = client.get_user_wallet_eligibility().await;
        match result {
            Ok(eligible) => {
                assert_eq!(eligible, 0);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
