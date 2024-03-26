use std::sync::Arc;

use andromeda_common::BitcoinUnit;
use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::Deserialize;

use crate::{error::Error, settings::FiatCurrency, BASE_WALLET_API_V1};

#[derive(Clone)]
pub struct ExchangeRateClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiExchangeRate {
    /// An encrypted ID
    pub ID: String,
    /// Bitcoin unit of the exchange rate
    pub BitcoinUnit: BitcoinUnit,
    /// Fiat currency of the exchange rate
    pub FiatCurrency: FiatCurrency,
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

impl ExchangeRateClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_exchange_rate(
        &self,
        fiat_currency: FiatCurrency,
        time: Option<u64>,
    ) -> Result<ApiExchangeRate, Error> {
        let path = match time {
            Some(t) => format!(
                "{}/rates?FiatCurrency={}&Time={}",
                BASE_WALLET_API_V1,
                fiat_currency.to_string(),
                t.to_string(),
            ),
            None => format!(
                "{}/rates?FiatCurrency={}",
                BASE_WALLET_API_V1,
                fiat_currency.to_string(),
            ),
        };
        let request = ProtonRequest::new(Method::GET, path);

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
            .to_json::<GetExchangeRateResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.ExchangeRate)
    }
}

#[cfg(test)]
mod tests {
    use super::ExchangeRateClient;
    use crate::{settings::FiatCurrency, utils::common_session};

    #[tokio::test]
    #[ignore]
    async fn should_get_exchange_rate() {
        let session = common_session().await;
        let client = ExchangeRateClient::new(session);

        let exchange_rate = client
            .get_exchange_rate(FiatCurrency::EUR, Some(1707287982))
            .await;

        println!("request done: {:?}", exchange_rate);
    }
}
