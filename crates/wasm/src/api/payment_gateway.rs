use andromeda_api::payment_gateway::{
    ApiCountry, ApiSimpleFiatCurrency, CountriesByProvider, FiatCurrenciesByProvider, GatewayProvider,
    PaymentGatewayClient, PaymentMethod, PaymentMethodsByProvider, Quote, QuotesByProvider,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPaymentGatewayClient(PaymentGatewayClient);

impl From<PaymentGatewayClient> for WasmPaymentGatewayClient {
    fn from(value: PaymentGatewayClient) -> Self {
        Self(value)
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmGatewayProvider {
    Banxa,
    Ramp,
    MoonPay,
    Unsupported,
}

impl From<GatewayProvider> for WasmGatewayProvider {
    fn from(value: GatewayProvider) -> Self {
        match value {
            GatewayProvider::Banxa => WasmGatewayProvider::Banxa,
            GatewayProvider::Ramp => WasmGatewayProvider::Ramp,
            GatewayProvider::MoonPay => WasmGatewayProvider::MoonPay,
            GatewayProvider::Unsupported => WasmGatewayProvider::Unsupported,
        }
    }
}

impl From<WasmGatewayProvider> for GatewayProvider {
    fn from(value: WasmGatewayProvider) -> Self {
        match value {
            WasmGatewayProvider::Banxa => GatewayProvider::Banxa,
            WasmGatewayProvider::Ramp => GatewayProvider::Ramp,
            WasmGatewayProvider::MoonPay => GatewayProvider::MoonPay,
            WasmGatewayProvider::Unsupported => GatewayProvider::Unsupported,
        }
    }
}

// Countries
#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiCountry {
    pub Code: String,
    pub FiatCurrency: String,
    pub Name: String,
}

impl From<&ApiCountry> for WasmApiCountry {
    fn from(value: &ApiCountry) -> Self {
        WasmApiCountry {
            Code: value.Code.clone(),
            FiatCurrency: value.FiatCurrency.clone(),
            Name: value.Name.clone(),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCountries {
    pub data: Vec<WasmApiCountry>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmCountriesAndProviderTupple(pub WasmGatewayProvider, pub WasmCountries);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmCountriesByProvider {
    pub data: Vec<WasmCountriesAndProviderTupple>,
}

impl From<CountriesByProvider> for WasmCountriesByProvider {
    fn from(value: CountriesByProvider) -> Self {
        let countries_and_provider_tupple = value
            .keys()
            .into_iter()
            .map(|provider| {
                let default = &Vec::new();
                let countries = value.get(provider).unwrap_or(default);
                WasmCountriesAndProviderTupple(
                    (*provider).into(),
                    WasmCountries {
                        data: countries.into_iter().map(|c| c.into()).collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        WasmCountriesByProvider {
            data: countries_and_provider_tupple,
        }
    }
}

// Fiat currencies
#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiSimpleFiatCurrency {
    pub Symbol: String,
    pub Name: String,
}

impl From<&ApiSimpleFiatCurrency> for WasmApiSimpleFiatCurrency {
    fn from(value: &ApiSimpleFiatCurrency) -> Self {
        WasmApiSimpleFiatCurrency {
            Symbol: value.Symbol.clone(),
            Name: value.Name.clone(),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmFiatCurrencies {
    pub data: Vec<WasmApiSimpleFiatCurrency>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmFiatCurrenciesAndProviderTupple(pub WasmGatewayProvider, pub WasmFiatCurrencies);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmFiatCurrenciesByProvider {
    pub data: Vec<WasmFiatCurrenciesAndProviderTupple>,
}

impl From<FiatCurrenciesByProvider> for WasmFiatCurrenciesByProvider {
    fn from(value: FiatCurrenciesByProvider) -> Self {
        let fiat_currency_and_provider_tupples = value
            .keys()
            .into_iter()
            .map(|provider| {
                let default = &Vec::new();
                let fiat_currencies = value.get(provider).unwrap_or(default);
                WasmFiatCurrenciesAndProviderTupple(
                    (*provider).into(),
                    WasmFiatCurrencies {
                        data: fiat_currencies.into_iter().map(|c| c.into()).collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        WasmFiatCurrenciesByProvider {
            data: fiat_currency_and_provider_tupples,
        }
    }
}

// Payment methods
#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub enum WasmPaymentMethod {
    ApplePay,
    BankTransfer,
    Card,
    GooglePay,
    InstantPayment,
    Unsupported,
}

impl From<&PaymentMethod> for WasmPaymentMethod {
    fn from(value: &PaymentMethod) -> Self {
        match value {
            PaymentMethod::ApplePay => WasmPaymentMethod::ApplePay,
            PaymentMethod::BankTransfer => WasmPaymentMethod::BankTransfer,
            PaymentMethod::Card => WasmPaymentMethod::Card,
            PaymentMethod::GooglePay => WasmPaymentMethod::GooglePay,
            PaymentMethod::InstantPayment => WasmPaymentMethod::InstantPayment,
            PaymentMethod::Unsupported => WasmPaymentMethod::Unsupported,
        }
    }
}

impl From<WasmPaymentMethod> for PaymentMethod {
    fn from(value: WasmPaymentMethod) -> Self {
        match value {
            WasmPaymentMethod::ApplePay => PaymentMethod::ApplePay,
            WasmPaymentMethod::BankTransfer => PaymentMethod::BankTransfer,
            WasmPaymentMethod::Card => PaymentMethod::Card,
            WasmPaymentMethod::GooglePay => PaymentMethod::GooglePay,
            WasmPaymentMethod::InstantPayment => PaymentMethod::InstantPayment,
            WasmPaymentMethod::Unsupported => PaymentMethod::Unsupported,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPaymentMethods {
    pub data: Vec<WasmPaymentMethod>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmPaymentMethodsAndProviderTupple(pub WasmGatewayProvider, pub WasmPaymentMethods);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmPaymentMethodsByProvider {
    pub data: Vec<WasmPaymentMethodsAndProviderTupple>,
}

impl From<PaymentMethodsByProvider> for WasmPaymentMethodsByProvider {
    fn from(value: PaymentMethodsByProvider) -> Self {
        let countries_and_provider_tupple = value
            .keys()
            .into_iter()
            .map(|provider| {
                let default = &Vec::new();
                let payment_methods = value.get(provider).unwrap_or(default);
                WasmPaymentMethodsAndProviderTupple(
                    (*provider).into(),
                    WasmPaymentMethods {
                        data: payment_methods.into_iter().map(|c| c.into()).collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        WasmPaymentMethodsByProvider {
            data: countries_and_provider_tupple,
        }
    }
}

// Quotes
#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmQuote {
    pub BitcoinAmount: String,
    pub FiatAmount: String,
    pub FiatCurrencySymbol: String,
    pub NetworkFee: String,
    pub PaymentGatewayFee: String,
    pub PaymentMethod: WasmPaymentMethod,
}

impl From<&Quote> for WasmQuote {
    fn from(value: &Quote) -> Self {
        WasmQuote {
            BitcoinAmount: value.BitcoinAmount.clone(),
            FiatAmount: value.FiatAmount.clone(),
            FiatCurrencySymbol: value.FiatCurrencySymbol.clone(),
            NetworkFee: value.NetworkFee.clone(),
            PaymentGatewayFee: value.PaymentGatewayFee.clone(),
            PaymentMethod: (&value.PaymentMethod).into(),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmQuotes {
    pub data: Vec<WasmQuote>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmQuotesAndProviderTupple(pub WasmGatewayProvider, pub WasmQuotes);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmQuotesByProvider {
    pub data: Vec<WasmQuotesAndProviderTupple>,
}

impl From<QuotesByProvider> for WasmQuotesByProvider {
    fn from(value: QuotesByProvider) -> Self {
        let countries_and_provider_tupple = value
            .keys()
            .into_iter()
            .map(|provider| {
                let default = &Vec::new();
                let quotes = value.get(provider).unwrap_or(default);
                WasmQuotesAndProviderTupple(
                    (*provider).into(),
                    WasmQuotes {
                        data: quotes.into_iter().map(|c| c.into()).collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        WasmQuotesByProvider {
            data: countries_and_provider_tupple,
        }
    }
}

#[wasm_bindgen]
impl WasmPaymentGatewayClient {
    #[wasm_bindgen(js_name = "getCountries")]
    pub async fn get_countries(&self) -> Result<WasmCountriesByProvider, js_sys::Error> {
        self.0
            .get_countries()
            .await
            .map(|c| c.into())
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getFiatCurrencies")]
    pub async fn get_fiat_currencies(&self) -> Result<WasmFiatCurrenciesByProvider, js_sys::Error> {
        self.0
            .get_fiat_currencies()
            .await
            .map(|c| c.into())
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getPaymentMethods")]
    pub async fn get_payment_methods(
        &self,
        fiat_currency: String,
    ) -> Result<WasmPaymentMethodsByProvider, js_sys::Error> {
        self.0
            .get_payment_methods(fiat_currency)
            .await
            .map(|c| c.into())
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getQuotes")]
    pub async fn get_quotes(
        &self,
        amount: f64,
        fiat_currency: String,
        payment_method: Option<WasmPaymentMethod>,
        provider: Option<WasmGatewayProvider>,
    ) -> Result<WasmQuotesByProvider, js_sys::Error> {
        self.0
            .get_quotes(
                amount,
                fiat_currency,
                payment_method.map(|p| p.into()),
                provider.map(|o| o.into()),
            )
            .await
            .map(|c| c.into())
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "createOnRampCheckout")]
    pub async fn create_on_ramp_checkout(
        &self,
        amount: String,
        btc_address: String,
        fiat_currency: String,
        payment_method: WasmPaymentMethod,
        provider: WasmGatewayProvider,
    ) -> Result<String, js_sys::Error> {
        self.0
            .create_on_ramp_checkout(
                amount,
                btc_address,
                fiat_currency,
                payment_method.into(),
                provider.into(),
            )
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "signUrl")]
    pub async fn sign_url(&self, url: String, provider: WasmGatewayProvider) -> Result<String, js_sys::Error> {
        self.0.sign_url(url, provider.into()).await.map_err(|e| e.to_js_error())
    }
}
