use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{
    request::{Method, ProtonRequest, Response},
    session::Session,
};
use serde::{Deserialize, Serialize};

use crate::{error::Error, BASE_WALLET_API_V1};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum BitcoinUnit {
    /// 100,000,000 sats
    BTC,
    /// 100,000 sats
    MBTC,
    /// 1 sat
    SAT,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum FiatCurrency {
    USD,
    EUR,
    CHF,
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
        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetUserSettingsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn bitcoin_unit(&self, symbol: BitcoinUnit) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/currency/bitcoin", BASE_WALLET_API_V1))
            .json_body(UpdateBitcoinUnitRequestBody { Symbol: symbol })
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetUserSettingsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn fiat_currency(&self, symbol: FiatCurrency) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/currency/fiat", BASE_WALLET_API_V1))
            .json_body(UpdateFiatCurrencyRequestBody { Symbol: symbol })
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetUserSettingsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn two_fa_threshold(&self, amount: u64) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/2fa/threshold", BASE_WALLET_API_V1))
            .json_body(Update2FAThresholdRequestBody {
                TwoFactorAmountThreshold: amount,
            })
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetUserSettingsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletUserSettings)
    }

    pub async fn hide_empty_used_addresses(&self, hide_empty_used_addresses: bool) -> Result<UserSettings, Error> {
        let request = ProtonRequest::new(
            Method::PUT,
            format!("{}/settings/addresses/used/hide", BASE_WALLET_API_V1),
        )
        .json_body(UpdateHideEmptyUsedAddressesRequestBody {
            HideEmptyUsedAddresses: hide_empty_used_addresses.into(),
        })
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetUserSettingsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletUserSettings)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::common_session;

    use super::SettingsClient;

    #[tokio::test]
    #[ignore]
    async fn should_get_network() {
        let session = common_session().await;
        let client = SettingsClient::new(session);

        let settings = client.get_user_settings().await;
        println!("request done: {:?}", settings);
    }
}
