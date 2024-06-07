use std::{collections::HashMap, fmt, sync::Arc};

use muon::Request;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
    error::Error,
    settings::FiatCurrencySymbol,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
pub enum GatewayProvider {
    Banxa,
    Ramp,
    MoonPay,
    // A fallback value to handle potential failures resulting from changes in the server value.
    #[serde(other)]
    Unsupported,
}

impl fmt::Display for GatewayProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// countries part
pub type CountriesByProvider = HashMap<GatewayProvider, Vec<ApiCountry>>;
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetCountriesResponseBody {
    pub Code: i32,
    pub Countries: CountriesByProvider,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ApiCountry {
    pub Code: String,
    pub FiatCurrency: String,
    pub Name: String,
}

/// fiat currencies part
pub type FiatCurrenciesByProvider = HashMap<GatewayProvider, Vec<ApiFiatCurrency>>;
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetFiatCurrenciesResponseBody {
    pub Code: i32,
    pub FiatCurrencies: FiatCurrenciesByProvider,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ApiFiatCurrency {
    pub Name: String,
    pub Symbol: String,
}

/// payment methods part
pub type PaymentMethodsByProvider = HashMap<GatewayProvider, Vec<PaymentMethod>>;
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetPaymentMethodsResponseBody {
    pub Code: i32,
    pub PaymentMethods: PaymentMethodsByProvider,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum PaymentMethod {
    ApplePay = 1,
    BankTransfer = 2,
    Card = 3,
    GooglePay = 4,
    InstantPayment = 5,
    #[serde(other)]
    Unsupported,
}

/// quotes part
pub type QuotesByProvider = HashMap<GatewayProvider, Vec<Quote>>;
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Quote {
    pub BitcoinAmount: String,
    pub FiatAmount: String,
    pub FiatCurrencySymbol: FiatCurrencySymbol,
    pub NetworkFee: String,
    pub PaymentGatewayFee: String,
    pub PaymentMethod: PaymentMethod,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetQuotesResponseBody {
    pub Code: i32,
    pub Quotes: QuotesByProvider,
}

#[derive(Clone)]
pub struct ProtonPaymentGatewayClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ProtonPaymentGatewayClient {
    pub fn new(api: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client: api }
    }

    pub async fn get_countries(&self) -> Result<CountriesByProvider, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "payment-gateway/countries")
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetCountriesResponseBody>()?;
        Ok(parsed.Countries)
    }

    pub async fn get_fiat_currencies(&self) -> Result<FiatCurrenciesByProvider, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "payment-gateway/fiats")
            .to_get_request();
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetFiatCurrenciesResponseBody>()?;
        Ok(parsed.FiatCurrencies)
    }

    pub async fn get_payment_methods(
        &self,
        fiat_symbol: FiatCurrencySymbol,
    ) -> Result<PaymentMethodsByProvider, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "payment-gateway/payment-methods")
            .to_get_request()
            .param("FiatCurrency", Some(fiat_symbol.to_string()));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetPaymentMethodsResponseBody>()?;
        Ok(parsed.PaymentMethods)
    }

    pub async fn get_quotes(
        &self,
        amount: String,
        fiat_currency: FiatCurrencySymbol,
        pay_method: PaymentMethod,
        provider: Option<GatewayProvider>,
    ) -> Result<QuotesByProvider, Error> {
        let pay_method = pay_method as i32;
        let mut request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "payment-gateway/quotes")
            .to_get_request()
            .param("Amount", Some(amount))
            .param("FiatCurrency", Some(fiat_currency.to_string()))
            .param("PaymentMethod", Some(pay_method.to_string()));
        if let Some(value) = provider {
            request = request.param("Provider", Some(value.to_string()));
        }
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetQuotesResponseBody>()?;
        Ok(parsed.Quotes)
    }
}

#[cfg(test)]
mod tests {

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        payment_gateway::{
            GatewayProvider, GetCountriesResponseBody, GetFiatCurrenciesResponseBody, GetPaymentMethodsResponseBody,
            GetQuotesResponseBody, PaymentMethod, ProtonPaymentGatewayClient,
        },
        settings::FiatCurrencySymbol,
        tests::utils::{common_api_client, setup_test_connection},
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    async fn test_get_countries_parse() {
        let json_data = r#"
        {
            "Code": 1000,
            "Countries": {
                "Banxa": [
                    {
                        "Code": "AU",
                        "FiatCurrency": "AUD",
                        "Name": "Australia"
                    }
                ],
                "Ramp": [
                    {
                        "Code": "AU",
                        "FiatCurrency": "AUD",
                        "Name": "Australia"
                    }
                ],
                "MoonPay": [
                    {
                        "Code": "AU",
                        "FiatCurrency": "AUD",
                        "Name": "Australia"
                    }
                ]
            }
        }"#;

        let response: GetCountriesResponseBody = serde_json::from_str(json_data).unwrap();
        assert!(response.Code == 1000);
        assert!(response.Countries.get(&GatewayProvider::Banxa).unwrap().len() == 1);
        assert!(response.Countries.get(&GatewayProvider::Banxa).unwrap()[0].Code == "AU");
        assert!(response.Countries.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australia");
        assert!(response.Countries.get(&GatewayProvider::Ramp).unwrap()[0].Code == "AU");
        assert!(response.Countries.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australia");
        assert!(response.Countries.get(&GatewayProvider::MoonPay).unwrap()[0].Code == "AU");
        assert!(response.Countries.get(&GatewayProvider::MoonPay).unwrap()[0].Name == "Australia");
    }

    #[tokio::test]
    async fn test_get_fiat_currencies_parse() {
        let json_value = serde_json::json!(
        {
            "Code": 1000,
            "FiatCurrencies": {
                "Banxa": [
                    {
                        "Name": "Australian Dollar",
                        "Symbol": "AUD"
                    }
                ],
                "Ramp": [
                    {
                        "Name": "Australian Dollar",
                        "Symbol": "AUD"
                    }
                ],
                "MoonPay": [
                    {
                        "Name": "Australian Dollar",
                        "Symbol": "AUD"
                    }
                ]
            }
        });

        let response: GetFiatCurrenciesResponseBody = serde_json::from_value(json_value).unwrap();
        assert!(response.Code == 1000);
        assert!(response.FiatCurrencies.get(&GatewayProvider::Banxa).unwrap().len() == 1);
        assert!(response.FiatCurrencies.get(&GatewayProvider::Banxa).unwrap()[0].Symbol == "AUD");
        assert!(response.FiatCurrencies.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australian Dollar");
        assert!(response.FiatCurrencies.get(&GatewayProvider::Ramp).unwrap()[0].Symbol == "AUD");
        assert!(response.FiatCurrencies.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australian Dollar");
        assert!(response.FiatCurrencies.get(&GatewayProvider::MoonPay).unwrap()[0].Symbol == "AUD");
        assert!(response.FiatCurrencies.get(&GatewayProvider::MoonPay).unwrap()[0].Name == "Australian Dollar");
    }

    #[tokio::test]
    async fn test_get_payment_methods_parse() {
        let json_value = serde_json::json!(
        {
            "Code": 1000,
            "PaymentMethods": {
                "Banxa": [1,2,3,4,8],
                "Ramp": [1,2,3],
                "MoonPay": [1,223,4,9,10],
                "Server": [1,2,3,4],
                "Change": [1,5],
            }
        });

        let check = serde_json::from_value::<GetPaymentMethodsResponseBody>(json_value).unwrap();
        assert!(check.PaymentMethods.get(&GatewayProvider::Banxa).unwrap().len() == 5);
        // hash map union the keys
        assert!(check.PaymentMethods.len() == 4);
        assert!(check.PaymentMethods.get(&GatewayProvider::Banxa).unwrap()[0] == PaymentMethod::ApplePay);
        assert!(check.PaymentMethods.get(&GatewayProvider::Ramp).unwrap()[2] == PaymentMethod::Card);
        let moonpay = check.PaymentMethods.get(&GatewayProvider::MoonPay).unwrap();
        assert!(moonpay[0] == PaymentMethod::ApplePay);
        assert!(moonpay[1] == PaymentMethod::Unsupported);
        assert!(moonpay[2] == PaymentMethod::GooglePay);
        assert!(moonpay[3] == PaymentMethod::Unsupported);
        assert!(moonpay[4] == PaymentMethod::Unsupported);
    }

    #[tokio::test]
    async fn test_get_payment_methods_parse_empty() {
        let json_value = serde_json::json!(
        {
            "Code": 1000,
            "PaymentMethods": {
                "Banxa": [],
                "Ramp": [],
                "When": [],
                "MoonPay": [],
                "Change": [],
            }
        });

        let check = serde_json::from_value::<GetPaymentMethodsResponseBody>(json_value).unwrap();
        // hash map union the keys
        assert!(check.PaymentMethods.len() == 4);
        assert!(check.PaymentMethods.get(&GatewayProvider::Banxa).unwrap().len() == 0);
        assert!(check.PaymentMethods.get(&GatewayProvider::Unsupported).unwrap().len() == 0);
        assert!(check.PaymentMethods.get(&GatewayProvider::Ramp).unwrap().len() == 0);
        assert!(check.PaymentMethods.get(&GatewayProvider::MoonPay).unwrap().len() == 0);
    }

    #[test]
    fn test_get_quotes_parse() {
        let json_data = r#"
        {
            "Code": 1000,
            "Quotes": {
                "Banxa": [
                    {
                        "BitcoinAmount": "0.00437556",
                        "FiatAmount": "300.50",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "1.34",
                        "PaymentGatewayFee": "5.85",
                        "PaymentMethod": 1
                    }
                ],
                "Ramp": [
                    {
                        "BitcoinAmount": "0.00437556",
                        "FiatAmount": "300.50",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "1.34",
                        "PaymentGatewayFee": "5.85",
                        "PaymentMethod": 1
                    }
                ],
                "MoonPay": [
                    {
                        "BitcoinAmount": "0.00437556",
                        "FiatAmount": "300.50",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "1.34",
                        "PaymentGatewayFee": "5.85",
                        "PaymentMethod": 1
                    }
                ]
            }
        }"#;

        let response: GetQuotesResponseBody = serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(response.Code, 1000);
        assert_eq!(
            response.Quotes[&GatewayProvider::Banxa][0].PaymentMethod,
            PaymentMethod::ApplePay
        );
        assert_eq!(
            response.Quotes[&GatewayProvider::Ramp][0].PaymentMethod,
            PaymentMethod::ApplePay
        );
        //
        assert_eq!(
            response.Quotes[&GatewayProvider::MoonPay][0].BitcoinAmount,
            "0.00437556"
        );
        let moonpay = response.Quotes.get(&GatewayProvider::MoonPay).unwrap();
        assert_eq!(moonpay[0].FiatAmount, "300.50");
        assert_eq!(moonpay[0].FiatCurrencySymbol, FiatCurrencySymbol::EUR);
        assert_eq!(moonpay[0].NetworkFee, "1.34");
        assert_eq!(moonpay[0].PaymentGatewayFee, "5.85",);
        assert_eq!(moonpay[0].PaymentMethod, PaymentMethod::ApplePay);
    }

    #[test]
    fn test_get_quotes_parse_empty() {
        let json_data = r#"
        {
            "Code": 1000,
            "Quotes": {}
        }"#;

        let response: GetQuotesResponseBody = serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(response.Code, 1000);
        assert_eq!(response.Quotes.len(), 0);
    }

    #[test]
    fn test_get_quotes_parse_provider_empty() {
        let json_data = r#"
        {
            "Code": 1000,
            "Quotes": {
                "Banxa": [],
                "Ramp": []
            }
        }"#;

        let response: GetQuotesResponseBody = serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(response.Code, 1000);
        assert_eq!(response.Quotes.len(), 2);
        assert_eq!(response.Quotes[&GatewayProvider::Ramp].len(), 0);
        assert_eq!(response.Quotes[&GatewayProvider::Banxa].len(), 0);
    }

    #[tokio::test]
    async fn test_get_countries_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Countries": {
                "Banxa": [
                    {
                        "Code": "AU",
                        "FiatCurrency": "AUD",
                        "Name": "Australia"
                    }
                ],
                "Ramp": [
                    {
                        "Code": "AU",
                        "FiatCurrency": "AUD",
                        "Name": "Australia"
                    }
                ]
            }
        });
        let req_path: String = format!("{}/payment-gateway/countries", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let gateway_client = ProtonPaymentGatewayClient::new(api_client);
        let countries = gateway_client.get_countries().await;
        match countries {
            Ok(value) => {
                assert!(value.len() > 0);
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Code == "AU");
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australia");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Code == "AU");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australia");
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e.to_string()),
        }
    }

    #[tokio::test]
    async fn test_get_fiat_currencies_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "FiatCurrencies": {
                "Banxa": [
                    {
                        "Name": "Australian Dollar",
                        "Symbol": "AUD"
                    }
                ],
                "Ramp": [
                    {
                        "Name": "Australian Dollar",
                        "Symbol": "AUD"
                    }
                ]
            }
        });

        let req_path: String = format!("{}/payment-gateway/fiats", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let gateway_client = ProtonPaymentGatewayClient::new(api_client);
        let fiat_currencies = gateway_client.get_fiat_currencies().await;
        match fiat_currencies {
            Ok(value) => {
                assert!(value.len() > 0);
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Symbol == "AUD");
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australian Dollar");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Symbol == "AUD");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australian Dollar");
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e.to_string()),
        }
    }

    #[tokio::test]
    async fn test_get_payment_methods_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "PaymentMethods": {
                "Banxa": [1],
                "Ramp": [6]
            }
        });
        let req_path: String = format!("{}/payment-gateway/payment-methods", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let gateway_client = ProtonPaymentGatewayClient::new(api_client);
        let fiat_currencies = gateway_client.get_payment_methods(FiatCurrencySymbol::AUD).await;
        match fiat_currencies {
            Ok(value) => {
                assert!(value.len() > 0);
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0] == PaymentMethod::ApplePay);
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0] == PaymentMethod::Unsupported);
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e.to_string()),
        }
    }

    #[tokio::test]
    async fn test_get_quotes_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Quotes": {
                "Banxa": [
                    {
                        "BitcoinAmount": "0.00437556",
                        "FiatAmount": "300.50",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "1.34",
                        "PaymentGatewayFee": "5.85",
                        "PaymentMethod": 1
                    }
                ],
                "Ramp": [
                    {
                        "BitcoinAmount": "0.00437556",
                        "FiatAmount": "300.50",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "1.34",
                        "PaymentGatewayFee": "5.85",
                        "PaymentMethod": 1
                    }
                ]
            }
        });
        let req_path: String = format!("{}/payment-gateway/quotes", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let gateway_client = ProtonPaymentGatewayClient::new(api_client);
        let quotes = gateway_client
            .get_quotes(
                "amount".to_string(),
                FiatCurrencySymbol::AUD,
                PaymentMethod::ApplePay,
                Some(GatewayProvider::Ramp),
            )
            .await;
        match quotes {
            Ok(value) => {
                assert!(value.len() > 0);
                assert!(value[&GatewayProvider::Banxa].len() == 1);
                assert!(value[&GatewayProvider::Banxa][0].BitcoinAmount == "0.00437556");
                assert_eq!(value[&GatewayProvider::Banxa][0].PaymentMethod, PaymentMethod::ApplePay);
                assert_eq!(value[&GatewayProvider::Banxa][0].FiatAmount, "300.50");
                assert_eq!(
                    value[&GatewayProvider::Banxa][0].FiatCurrencySymbol,
                    FiatCurrencySymbol::EUR
                );
                assert_eq!(value[&GatewayProvider::Banxa][0].NetworkFee, "1.34");
                assert_eq!(value[&GatewayProvider::Banxa][0].PaymentGatewayFee, "5.85");

                assert!(value[&GatewayProvider::Ramp][0].BitcoinAmount == "0.00437556");
                assert_eq!(value[&GatewayProvider::Ramp][0].PaymentMethod, PaymentMethod::ApplePay);
                assert_eq!(value[&GatewayProvider::Ramp][0].FiatAmount, "300.50");
                assert_eq!(
                    value[&GatewayProvider::Ramp][0].FiatCurrencySymbol,
                    FiatCurrencySymbol::EUR
                );
                assert_eq!(value[&GatewayProvider::Ramp][0].NetworkFee, "1.34");
                assert_eq!(value[&GatewayProvider::Ramp][0].PaymentGatewayFee, "5.85");
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e.to_string()),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_countries() {
        let api_client = common_api_client().await;
        let client = ProtonPaymentGatewayClient::new(api_client);
        let countries = client.get_countries().await.unwrap();
        println!("get_countries done: {:?}", countries);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_fiat_currencies() {
        let api_client = common_api_client().await;
        let client = ProtonPaymentGatewayClient::new(api_client);
        let fiat_currencies = client.get_fiat_currencies().await.unwrap();
        println!("get_fiat_currencies done: {:?}", fiat_currencies);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_payment_methods() {
        let api_client = common_api_client().await;
        let client = ProtonPaymentGatewayClient::new(api_client);

        let payments = client.get_payment_methods(FiatCurrencySymbol::AUD).await.unwrap();
        println!("get_payment_methods done: {:?}", payments);
    }
    #[tokio::test]
    #[ignore]
    async fn should_get_quotes() {
        let api_client = common_api_client().await;
        let client = ProtonPaymentGatewayClient::new(api_client);

        let quotes = client
            .get_quotes(
                "2.00".to_string(),
                FiatCurrencySymbol::AUD,
                PaymentMethod::ApplePay,
                Some(GatewayProvider::Ramp),
            )
            .await
            .unwrap();
        println!("get_quotes done: {:?}", quotes);
    }
}
