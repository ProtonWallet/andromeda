use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    settings::FiatCurrencySymbol,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Clone, Debug, Deserialize)]
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
        let mut request = self.get("rates").query(("FiatCurrency", fiat_currency.to_string()));
        if let Some(time) = time {
            request = request.query(("Time", time.to_string()))
        }

        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetExchangeRateResponseBody>()?;
        Ok(parsed.ExchangeRate)
    }

    pub async fn get_all_fiat_currencies(&self) -> Result<Vec<ApiFiatCurrency>, Error> {
        let request = self.get("fiat-currencies");

        let response = self.api_client.send(request).await?;

        let parsed = response.parse_response::<GetAllFiatCurrenciesResponseBody>()?;
        Ok(parsed.FiatCurrencies)
    }
}

#[cfg(test)]
mod tests {
    use super::ExchangeRateClient;
    use crate::{
        core::ApiClient, settings::FiatCurrencySymbol, tests::utils::common_api_client,
        tests::utils::setup_test_connection, BASE_WALLET_API_V1,
    };
    use andromeda_common::BitcoinUnit;
    use wiremock::{
        matchers::{method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

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

    #[tokio::test]
    async fn test_get_exchange_rate_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "ExchangeRate": {
                    "ID": "BG2rHbE0giOBTvPWDVHdS_MMyxemjRxSzrKOTbxaINTH0zYnS5hD5zEqV9TURB-mzMy2LPC3qg4XnPq_kHmf9g==",
                    "BitcoinUnit": "BTC",
                    "FiatCurrency": "USD",
                    "Sign": "$",
                    "ExchangeRateTime": "1732266518",
                    "ExchangeRate": 9890500,
                    "Cents": 100
                }
            }
        );
        let fiat_currency = FiatCurrencySymbol::USD;
        let req_path: String = format!("{}/rates", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("FiatCurrency", fiat_currency.to_string()))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ExchangeRateClient::new(api_client);
        let result = client.get_exchange_rate(fiat_currency, None).await;
        match result {
            Ok(exchange_rate) => {
                assert_eq!(
                    exchange_rate.ID,
                    "BG2rHbE0giOBTvPWDVHdS_MMyxemjRxSzrKOTbxaINTH0zYnS5hD5zEqV9TURB-mzMy2LPC3qg4XnPq_kHmf9g=="
                );
                assert_eq!(exchange_rate.BitcoinUnit, BitcoinUnit::BTC);
                assert_eq!(exchange_rate.FiatCurrency, FiatCurrencySymbol::USD);
                assert_eq!(exchange_rate.Sign, Some("$".to_string()));
                assert_eq!(exchange_rate.ExchangeRateTime, "1732266518");
                assert_eq!(exchange_rate.ExchangeRate, 9890500);
                assert_eq!(exchange_rate.Cents, 100);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_all_fiat_currencies_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "FiatCurrencies": [
                    {
                    "ID": "FiatCurrency_001",
                    "Name": "Swiss Franc",
                    "Symbol": "CHF",
                    "Sign": "CHF",
                    "Cents": 100
                    },
                    {
                    "ID": "FiatCurrency_002",
                    "Name": "United Dollar",
                    "Symbol": "USD",
                    "Sign": "$",
                    "Cents": 100
                    }
                ]
            }
        );
        let req_path: String = format!("{}/fiat-currencies", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ExchangeRateClient::new(api_client);
        let result = client.get_all_fiat_currencies().await;
        match result {
            Ok(fiat_currencies) => {
                assert_eq!(fiat_currencies[0].ID, "FiatCurrency_001");
                assert_eq!(fiat_currencies[0].Name, "Swiss Franc");
                assert_eq!(fiat_currencies[0].Symbol, FiatCurrencySymbol::CHF);
                assert_eq!(fiat_currencies[0].Sign, "CHF");
                assert_eq!(fiat_currencies[0].Cents, 100);

                assert_eq!(fiat_currencies[1].ID, "FiatCurrency_002");
                assert_eq!(fiat_currencies[1].Name, "United Dollar");
                assert_eq!(fiat_currencies[1].Symbol, FiatCurrencySymbol::USD);
                assert_eq!(fiat_currencies[1].Sign, "$");
                assert_eq!(fiat_currencies[1].Cents, 100);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
