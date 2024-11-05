use std::{str, sync::Arc};

use andromeda_common::BitcoinUnit;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    settings::FiatCurrencySymbol,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};
use muon::common::ServiceType;

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Timeframe {
    OneDay = 1,
    OneWeek = 2,
    OneMonth = 3,
    // A fallback value to handle potential failures resulting from changes in the server value.
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct DataPoint {
    pub ExchangeRate: u64,
    pub Cents: u8,
    pub Timestamp: u64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct PriceGraph {
    pub FiatCurrency: FiatCurrencySymbol,
    pub BitcoinUnit: BitcoinUnit,
    pub GraphData: Vec<DataPoint>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetGraphDataResponseBody {
    pub Code: i32,
    pub PriceGraph: PriceGraph,
}

#[derive(Clone)]
pub struct PriceGraphClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for PriceGraphClient {
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

impl PriceGraphClient {
    pub async fn get_graph_data(
        &self,
        fiat_currency: FiatCurrencySymbol,
        timeframe: Timeframe,
    ) -> Result<PriceGraph, Error> {
        let request = self
            .get("graph")
            .query(("FiatCurrency", fiat_currency))
            .query(("Type", (timeframe as u8).to_string()))
            .service_type(ServiceType::Normal, true);

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetGraphDataResponseBody>()?;
        Ok(parsed.PriceGraph)
    }
}

#[cfg(test)]
mod tests {
    use andromeda_common::BitcoinUnit;
    use wiremock::{
        matchers::{method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        core::ApiClient,
        price_graph::{DataPoint, PriceGraph, PriceGraphClient, Timeframe},
        settings::FiatCurrencySymbol,
        tests::utils::setup_test_connection_arc,
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    async fn test_get_price_data() {
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "PriceGraph": {
                "FiatCurrency": "EUR",
                "BitcoinUnit": "BTC",
                "GraphData": [
                    {
                        "ExchangeRate": 6189900,
                        "Cents": 100,
                        "Timestamp": 1721632020
                    },
                    {
                        "ExchangeRate": 6170200,
                        "Cents": 100,
                        "Timestamp": 1721635680
                    },
                    {
                        "ExchangeRate": 6171400,
                        "Cents": 100,
                        "Timestamp": 1721639340
                    },
                    {
                        "ExchangeRate": 6190200,
                        "Cents": 100,
                        "Timestamp": 1721643000
                    },
                    {
                        "ExchangeRate": 6183400,
                        "Cents": 100,
                        "Timestamp": 1721646660
                    },
                    {
                        "ExchangeRate": 6195100,
                        "Cents": 100,
                        "Timestamp": 1721650320
                    },
                    {
                        "ExchangeRate": 6236800,
                        "Cents": 100,
                        "Timestamp": 1721653980
                    },
                    {
                        "ExchangeRate": 6182500,
                        "Cents": 100,
                        "Timestamp": 1721657640
                    },
                    {
                        "ExchangeRate": 6132200,
                        "Cents": 100,
                        "Timestamp": 1721661300
                    },
                    {
                        "ExchangeRate": 6171500,
                        "Cents": 100,
                        "Timestamp": 1721664960
                    },
                    {
                        "ExchangeRate": 6164900,
                        "Cents": 100,
                        "Timestamp": 1721668620
                    },
                    {
                        "ExchangeRate": 6199300,
                        "Cents": 100,
                        "Timestamp": 1721672280
                    },
                    {
                        "ExchangeRate": 6218400,
                        "Cents": 100,
                        "Timestamp": 1721675940
                    },
                    {
                        "ExchangeRate": 6247800,
                        "Cents": 100,
                        "Timestamp": 1721679600
                    },
                    {
                        "ExchangeRate": 6236400,
                        "Cents": 100,
                        "Timestamp": 1721683260
                    },
                    {
                        "ExchangeRate": 6203700,
                        "Cents": 100,
                        "Timestamp": 1721686920
                    },
                    {
                        "ExchangeRate": 6190700,
                        "Cents": 100,
                        "Timestamp": 1721690580
                    },
                    {
                        "ExchangeRate": 6202400,
                        "Cents": 100,
                        "Timestamp": 1721692800
                    },
                    {
                        "ExchangeRate": 6203400,
                        "Cents": 100,
                        "Timestamp": 1721696460
                    },
                    {
                        "ExchangeRate": 6190500,
                        "Cents": 100,
                        "Timestamp": 1721700120
                    },
                    {
                        "ExchangeRate": 6214100,
                        "Cents": 100,
                        "Timestamp": 1721703780
                    },
                    {
                        "ExchangeRate": 6193900,
                        "Cents": 100,
                        "Timestamp": 1721707440
                    },
                    {
                        "ExchangeRate": 6141500,
                        "Cents": 100,
                        "Timestamp": 1721711100
                    },
                    {
                        "ExchangeRate": 6125900,
                        "Cents": 100,
                        "Timestamp": 1721714760
                    },
                    {
                        "ExchangeRate": 6111600,
                        "Cents": 100,
                        "Timestamp": 1721718420
                    },
                    {
                        "ExchangeRate": 6111600,
                        "Cents": 100,
                        "Timestamp": 1721718194
                    }
                ]
            }
        });

        let req_path: String = format!("{}/graph", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("FiatCurrency", "EUR"))
            .and(query_param("Type", "1"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let gateway_client = PriceGraphClient::new(api_client);
        let graph_data = gateway_client
            .get_graph_data(FiatCurrencySymbol::EUR, Timeframe::OneDay)
            .await;

        println!("graph_datagraph_datagraph_data:{:?}", graph_data);
        assert_eq!(
            graph_data.unwrap(),
            PriceGraph {
                FiatCurrency: FiatCurrencySymbol::EUR,
                BitcoinUnit: BitcoinUnit::BTC,
                GraphData: vec![
                    DataPoint {
                        ExchangeRate: 6189900,
                        Cents: 100,
                        Timestamp: 1721632020
                    },
                    DataPoint {
                        ExchangeRate: 6170200,
                        Cents: 100,
                        Timestamp: 1721635680
                    },
                    DataPoint {
                        ExchangeRate: 6171400,
                        Cents: 100,
                        Timestamp: 1721639340
                    },
                    DataPoint {
                        ExchangeRate: 6190200,
                        Cents: 100,
                        Timestamp: 1721643000
                    },
                    DataPoint {
                        ExchangeRate: 6183400,
                        Cents: 100,
                        Timestamp: 1721646660
                    },
                    DataPoint {
                        ExchangeRate: 6195100,
                        Cents: 100,
                        Timestamp: 1721650320
                    },
                    DataPoint {
                        ExchangeRate: 6236800,
                        Cents: 100,
                        Timestamp: 1721653980
                    },
                    DataPoint {
                        ExchangeRate: 6182500,
                        Cents: 100,
                        Timestamp: 1721657640
                    },
                    DataPoint {
                        ExchangeRate: 6132200,
                        Cents: 100,
                        Timestamp: 1721661300
                    },
                    DataPoint {
                        ExchangeRate: 6171500,
                        Cents: 100,
                        Timestamp: 1721664960
                    },
                    DataPoint {
                        ExchangeRate: 6164900,
                        Cents: 100,
                        Timestamp: 1721668620
                    },
                    DataPoint {
                        ExchangeRate: 6199300,
                        Cents: 100,
                        Timestamp: 1721672280
                    },
                    DataPoint {
                        ExchangeRate: 6218400,
                        Cents: 100,
                        Timestamp: 1721675940
                    },
                    DataPoint {
                        ExchangeRate: 6247800,
                        Cents: 100,
                        Timestamp: 1721679600
                    },
                    DataPoint {
                        ExchangeRate: 6236400,
                        Cents: 100,
                        Timestamp: 1721683260
                    },
                    DataPoint {
                        ExchangeRate: 6203700,
                        Cents: 100,
                        Timestamp: 1721686920
                    },
                    DataPoint {
                        ExchangeRate: 6190700,
                        Cents: 100,
                        Timestamp: 1721690580
                    },
                    DataPoint {
                        ExchangeRate: 6202400,
                        Cents: 100,
                        Timestamp: 1721692800
                    },
                    DataPoint {
                        ExchangeRate: 6203400,
                        Cents: 100,
                        Timestamp: 1721696460
                    },
                    DataPoint {
                        ExchangeRate: 6190500,
                        Cents: 100,
                        Timestamp: 1721700120
                    },
                    DataPoint {
                        ExchangeRate: 6214100,
                        Cents: 100,
                        Timestamp: 1721703780
                    },
                    DataPoint {
                        ExchangeRate: 6193900,
                        Cents: 100,
                        Timestamp: 1721707440
                    },
                    DataPoint {
                        ExchangeRate: 6141500,
                        Cents: 100,
                        Timestamp: 1721711100
                    },
                    DataPoint {
                        ExchangeRate: 6125900,
                        Cents: 100,
                        Timestamp: 1721714760
                    },
                    DataPoint {
                        ExchangeRate: 6111600,
                        Cents: 100,
                        Timestamp: 1721718420
                    },
                    DataPoint {
                        ExchangeRate: 6111600,
                        Cents: 100,
                        Timestamp: 1721718194
                    }
                ]
            }
        )
    }
}
