use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    settings::FiatCurrencySymbol,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};
use muon::common::ServiceType;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiExchangeRate {
    /// An encrypted ID
    pub ID: String,
    /// Bitcoin unit of the exchange rate
    pub BitcoinUnit: BitcoinUnit,
    /// Fiat currency of the exchange rate
    pub FiatCurrency: FiatCurrencySymbol,
    /// Sign of the fiat currency (e.g. € for EUR)
    pub Sign: Option<String>,
    /// string <date-time>
    pub ExchangeRateTime: String,
    /// Exchange rate BitcoinUnit/FiatCurrency
    pub ExchangeRate: u64,
    /// Cents precision of the fiat currency (e.g. 1 for JPY, 100 for USD)
    pub Cents: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetExchangeRateResponseBody {
    //TODO:: code need to be used. remove all #[allow(dead_code)]
    #[allow(dead_code)]
    pub Code: u16,
    pub ExchangeRate: ApiExchangeRate,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiFiatCurrency {
    pub ID: String,
    pub Name: String,
    pub Symbol: FiatCurrencySymbol,
    pub Sign: String,
    pub Cents: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetAllFiatCurrenciesResponseBody {
    //TODO:: code need to be used. remove all #[allow(dead_code)]
    #[allow(dead_code)]
    pub Code: u16,
    pub FiatCurrencies: Vec<ApiFiatCurrency>,
}

#[derive(Clone)]
pub struct ExchangeRateClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ExchangeRateClient {
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

impl ExchangeRateClient {
    pub async fn get_exchange_rate(
        &self,
        fiat_currency: FiatCurrencySymbol,
        time: Option<u64>,
    ) -> Result<ApiExchangeRate, Error> {
        let mut request = self
            .get("rates")
            .query(("FiatCurrency", fiat_currency.to_string()))
            .service_type(ServiceType::Normal, true);
        if let Some(time) = time {
            request = request.query(("Time", time.to_string()))
        }

        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetExchangeRateResponseBody>()?;
        Ok(parsed.ExchangeRate)
    }

    pub async fn get_all_fiat_currencies(&self) -> Result<Vec<ApiFiatCurrency>, Error> {
        let request = self.get("fiat-currencies").service_type(ServiceType::Normal, true);

        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetAllFiatCurrenciesResponseBody>()?;
        Ok(parsed.FiatCurrencies)
    }
}

#[cfg(test)]
mod tests {
    use super::ExchangeRateClient;
    use crate::{core::ApiClient, settings::FiatCurrencySymbol, tests::utils::common_api_client};

    #[tokio::test]
    #[ignore]
    async fn should_get_exchange_rate() {
        let api_client = common_api_client().await;
        let client = ExchangeRateClient::new(api_client);

        let exchange_rate = client
            .get_exchange_rate(FiatCurrencySymbol::EUR, Some(1707287982))
            .await;

        println!("request done: {:?}", exchange_rate);
        assert!(exchange_rate.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_all_fiat_currencies() {
        let api_client = common_api_client().await;
        let client = ExchangeRateClient::new(api_client);

        let fiat_currencies = client.get_all_fiat_currencies().await;

        println!("request done: {:?}", fiat_currencies);
        assert!(fiat_currencies.is_ok());
    }
}
