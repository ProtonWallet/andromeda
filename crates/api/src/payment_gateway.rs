use std::{collections::HashMap, fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
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

pub type FiatCurrenciesByProvider = HashMap<GatewayProvider, Vec<ApiSimpleFiatCurrency>>;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetFiatCurrenciesResponseBody {
    pub Code: i32,
    pub FiatCurrencies: FiatCurrenciesByProvider,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ApiSimpleFiatCurrency {
    pub Name: String,
    pub Symbol: String,
}

pub type PaymentMethodsByProvider = HashMap<GatewayProvider, Vec<PaymentMethod>>;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetPaymentMethodsResponseBody {
    pub Code: i32,
    pub PaymentMethods: PaymentMethodsByProvider,
}

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
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

pub type QuotesByProvider = HashMap<GatewayProvider, Vec<Quote>>;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Quote {
    pub BitcoinAmount: String,
    pub FiatAmount: String,
    pub FiatCurrencySymbol: String,
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

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct CreateOnRampCheckoutRequestBody {
    pub Amount: String,
    pub BitcoinAddress: String,
    pub FiatCurrency: String,
    pub PaymentMethod: PaymentMethod,
    pub Provider: GatewayProvider,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct CreateOnRampCheckoutResponseBody {
    pub Code: i32,
    pub CheckoutUrl: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SignUrlRequestBody {
    pub Url: String,
    pub Provider: GatewayProvider,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct SignUrlResponseBody {
    pub Code: i32,
    pub UrlSignature: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetPublicAPIKeyResponseBody {
    pub Code: i32,
    pub PublicApiKey: String,
}

#[derive(Clone)]
pub struct PaymentGatewayClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for PaymentGatewayClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl PaymentGatewayClient {
    pub async fn get_countries(&self) -> Result<CountriesByProvider, Error> {
        let request = self.get("payment-gateway/on-ramp/countries");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetCountriesResponseBody>()?;
        Ok(parsed.Countries)
    }

    pub async fn get_fiat_currencies(&self) -> Result<FiatCurrenciesByProvider, Error> {
        let request = self.get("payment-gateway/on-ramp/fiats");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetFiatCurrenciesResponseBody>()?;
        Ok(parsed.FiatCurrencies)
    }

    pub async fn get_payment_methods(&self, fiat_symbol: String) -> Result<PaymentMethodsByProvider, Error> {
        let request = self
            .get("payment-gateway/on-ramp/payment-methods")
            .query(("FiatCurrency", fiat_symbol.to_string()));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetPaymentMethodsResponseBody>()?;
        Ok(parsed.PaymentMethods)
    }

    pub async fn get_quotes(
        &self,
        amount: f64,
        fiat_currency: String,
        payment_method: Option<PaymentMethod>,
        provider: Option<GatewayProvider>,
    ) -> Result<QuotesByProvider, Error> {
        let mut request = self
            .get("payment-gateway/on-ramp/quotes")
            .query(("Amount", amount))
            .query(("FiatCurrency", fiat_currency.to_string()));

        if let Some(value) = payment_method {
            request = request.query(("PaymentMethod", (value as i32).to_string()));
        }

        if let Some(value) = provider {
            request = request.query(("Provider", value.to_string()));
        }

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetQuotesResponseBody>()?;
        Ok(parsed.Quotes)
    }

    pub async fn create_on_ramp_checkout(
        &self,
        amount: String,
        btc_address: String,
        fiat_currency: String,
        pay_method: PaymentMethod,
        provider: GatewayProvider,
    ) -> Result<String, Error> {
        let body = CreateOnRampCheckoutRequestBody {
            Amount: amount,
            BitcoinAddress: btc_address,
            FiatCurrency: fiat_currency,
            PaymentMethod: pay_method,
            Provider: provider,
        };
        let request = self.post("payment-gateway/on-ramp/checkout-url").body_json(body)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<CreateOnRampCheckoutResponseBody>()?;
        Ok(parsed.CheckoutUrl)
    }

    /// We expect only MoonPay to be provided as provider here, any other
    /// provider will return a 400 for now
    pub async fn sign_url(&self, url: String, provider: GatewayProvider) -> Result<String, Error> {
        let body = SignUrlRequestBody {
            Url: url,
            Provider: provider,
        };
        let request = self.post("payment-gateway/on-ramp/sign-url").body_json(body)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<SignUrlResponseBody>()?;

        Ok(parsed.UrlSignature)
    }

    /// We expect only MoonPay or Ramp to be provided as provider here, any
    /// other provider will return a 400 for now because they don't have any
    /// public api key to be returned
    pub async fn get_public_api_key(&self, provider: GatewayProvider) -> Result<String, Error> {
        let request = self
            .get("payment-gateway/on-ramp/public-api-key")
            .query(("Provider", provider));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetPublicAPIKeyResponseBody>()?;

        Ok(parsed.PublicApiKey)
    }
}

#[cfg(test)]
mod tests {

    use wiremock::{
        matchers::{body_json, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        core::ApiClient,
        payment_gateway::{
            GatewayProvider, GetCountriesResponseBody, GetFiatCurrenciesResponseBody, GetPaymentMethodsResponseBody,
            GetQuotesResponseBody, PaymentGatewayClient, PaymentMethod, SignUrlRequestBody,
        },
        tests::utils::{common_api_client, setup_test_connection_arc},
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
        assert!(check.PaymentMethods.get(&GatewayProvider::Banxa).unwrap().is_empty());
        assert!(check
            .PaymentMethods
            .get(&GatewayProvider::Unsupported)
            .unwrap()
            .is_empty());
        assert!(check.PaymentMethods.get(&GatewayProvider::Ramp).unwrap().is_empty());
        assert!(check.PaymentMethods.get(&GatewayProvider::MoonPay).unwrap().is_empty());
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
        assert_eq!(moonpay[0].FiatCurrencySymbol, "EUR");
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
        let req_path: String = format!("{}/payment-gateway/on-ramp/countries", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let countries = gateway_client.get_countries().await;
        match countries {
            Ok(value) => {
                assert!(!value.is_empty());
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Code == "AU");
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australia");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Code == "AU");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australia");
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
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

        let req_path: String = format!("{}/payment-gateway/on-ramp/fiats", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let fiat_currencies = gateway_client.get_fiat_currencies().await;
        match fiat_currencies {
            Ok(value) => {
                assert!(!value.is_empty());
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Symbol == "AUD");
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0].Name == "Australian Dollar");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Symbol == "AUD");
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0].Name == "Australian Dollar");
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
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
        let req_path: String = format!("{}/payment-gateway/on-ramp/payment-methods", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let fiat_currencies = gateway_client.get_payment_methods("AUD".to_string()).await;
        match fiat_currencies {
            Ok(value) => {
                assert!(!value.is_empty());
                assert!(value.get(&GatewayProvider::Banxa).unwrap().len() == 1);
                assert!(value.get(&GatewayProvider::Banxa).unwrap()[0] == PaymentMethod::ApplePay);
                assert!(value.get(&GatewayProvider::Ramp).unwrap()[0] == PaymentMethod::Unsupported);
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
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
                        "BitcoinAmount": "0.00135719",
                        "FiatAmount": "100",
                        "FiatCurrencySymbol": "EUR",
                        "NetworkFee": "2.0715229730449",
                        "PaymentGatewayFee": "3.3024386838943",
                        "PaymentMethod": 3,
                    }
                ]
            }
        });
        let req_path: String = format!("{}/payment-gateway/on-ramp/quotes", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Amount", "300.5"))
            .and(query_param("FiatCurrency", "AUD"))
            .and(query_param("PaymentMethod", "1"))
            .and(query_param("Provider", "Ramp"))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let quotes = gateway_client
            .get_quotes(
                300.50,
                "AUD".to_string(),
                Some(PaymentMethod::ApplePay),
                Some(GatewayProvider::Ramp),
            )
            .await;
        match quotes {
            Ok(value) => {
                assert!(!value.is_empty());
                assert!(value[&GatewayProvider::Banxa].len() == 1);
                assert!(value[&GatewayProvider::Banxa][0].BitcoinAmount == "0.00437556");
                assert_eq!(value[&GatewayProvider::Banxa][0].PaymentMethod, PaymentMethod::ApplePay);
                assert_eq!(value[&GatewayProvider::Banxa][0].FiatAmount, "300.50");
                assert_eq!(value[&GatewayProvider::Banxa][0].FiatCurrencySymbol, "EUR");
                assert_eq!(value[&GatewayProvider::Banxa][0].NetworkFee, "1.34");
                assert_eq!(value[&GatewayProvider::Banxa][0].PaymentGatewayFee, "5.85");

                assert!(value[&GatewayProvider::Ramp][0].BitcoinAmount == "0.00135719");
                assert_eq!(value[&GatewayProvider::Ramp][0].PaymentMethod, PaymentMethod::Card);
                assert_eq!(value[&GatewayProvider::Ramp][0].FiatAmount, "100");
                assert_eq!(value[&GatewayProvider::Ramp][0].FiatCurrencySymbol, "EUR");
                assert_eq!(value[&GatewayProvider::Ramp][0].NetworkFee, "2.0715229730449");
                assert_eq!(value[&GatewayProvider::Ramp][0].PaymentGatewayFee, "3.3024386838943");
            }

            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
        }
    }

    #[tokio::test]
    async fn test_create_on_ramp_checkout_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "CheckoutUrl": "https://example.com",
        });
        let req_path: String = format!("{}/payment-gateway/on-ramp/checkout-url", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "Amount": "10.00",
                "BitcoinAddress": "tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6",
                "FiatCurrency": "AUD",
                "PaymentMethod": 3,
                "Provider": "Banxa"
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(404))
            .with_priority(2)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let res = gateway_client
            .create_on_ramp_checkout(
                "10.00".to_string(),
                "tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6".to_string(),
                "AUD".to_string(),
                PaymentMethod::Card,
                GatewayProvider::Banxa,
            )
            .await;
        println!("create_on_ramp_checkout done: {:?}", res);
        match res {
            Ok(value) => assert!(!value.is_empty()),
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
        }

        let unmatched_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(unmatched_requests.len(), 1, "There should be no unmatched requests");
    }

    #[tokio::test]
    async fn test_create_on_ramp_checkout_1000_empty() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "CheckoutUrl": null,
        });
        let req_path: String = format!("{}/payment-gateway/on-ramp/checkout-url", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let res = gateway_client
            .create_on_ramp_checkout(
                "10.00".to_string(),
                "tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6".to_string(),
                "AUD".to_string(),
                PaymentMethod::Card,
                GatewayProvider::Banxa,
            )
            .await;
        match res {
            Ok(value) => panic!("Expected Error variant but got Ok.{}", value),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[tokio::test]
    async fn test_sign_url_1000() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "UrlSignature": "xyz",
        });
        let req_path: String = format!("{}/payment-gateway/on-ramp/sign-url", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(SignUrlRequestBody {
                Url: "https://example.com".to_string(),
                Provider: GatewayProvider::MoonPay,
            }))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let res = gateway_client
            .sign_url("https://example.com".to_string(), GatewayProvider::MoonPay)
            .await
            .unwrap();

        assert_eq!(res, "xyz".to_string())
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_countries() {
        let api_client = common_api_client().await;
        let client = PaymentGatewayClient::new(api_client);
        let countries = client.get_countries().await;
        println!("get_countries done: {:?}", countries);
        assert!(countries.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_fiat_currencies() {
        let api_client = common_api_client().await;
        let client = PaymentGatewayClient::new(api_client);
        let fiat_currencies = client.get_fiat_currencies().await;
        println!("get_fiat_currencies done: {:?}", fiat_currencies);
        assert!(fiat_currencies.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_payment_methods() {
        let api_client = common_api_client().await;
        let client = PaymentGatewayClient::new(api_client);

        let payments = client.get_payment_methods("AUD".to_string()).await;
        println!("get_payment_methods done: {:?}", payments);
        assert!(payments.is_ok());
    }
    #[tokio::test]
    #[ignore]
    async fn should_get_quotes() {
        let api_client = common_api_client().await;
        let client = PaymentGatewayClient::new(api_client);

        let quotes = client
            .get_quotes(
                2.00,
                "AUD".to_string(),
                Some(PaymentMethod::ApplePay),
                Some(GatewayProvider::Ramp),
            )
            .await;
        println!("get_quotes done: {:?}", quotes);
        assert!(quotes.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_on_ramp_checkout() {
        let api_client = common_api_client().await;
        let client = PaymentGatewayClient::new(api_client);

        let res = client
            .create_on_ramp_checkout(
                "10.00".to_string(),
                "tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6".to_string(),
                "AUD".to_string(),
                PaymentMethod::Card,
                GatewayProvider::Banxa,
            )
            .await;
        println!("create_on_ramp_checkout done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_public_api_key() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "PublicApiKey": "ABC"
        });
        let req_path: String = format!("{}/payment-gateway/on-ramp/public-api-key", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);

        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Provider", "MoonPay"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PaymentGatewayClient::new(api_client);
        let public_api_key = gateway_client
            .get_public_api_key(GatewayProvider::MoonPay)
            .await
            .unwrap();

        assert_eq!(public_api_key, "ABC");
    }
}
